use crate::prelude::*;
use mlua::prelude::*;

use crossbeam_channel::{bounded, Sender, Receiver};
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::fmt::Debug;

#[cfg(unix)]
use nix::fcntl::{fcntl, FcntlArg, OFlag};
#[cfg(unix)]
use std::os::unix::io::AsFd;

#[cfg(unix)]
pub fn make_nonblocking_unix<F: AsFd>(handle: F) -> std::io::Result<()> {
    // Get current flags
    let current_flags = fcntl(&handle, FcntlArg::F_GETFL)
        .map_err(|e| std::io::Error::from_raw_os_error(e as i32))?;

    // Set O_NONBLOCK
    let new_flags = OFlag::from_bits_truncate(current_flags) | OFlag::O_NONBLOCK;
    fcntl(handle, FcntlArg::F_SETFL(new_flags))
        .map_err(|e| std::io::Error::from_raw_os_error(e as i32))?;

    Ok(())
}

#[cfg(windows)]
use std::os::windows::io::AsRawHandle;
#[cfg(windows)]
use windows_sys::Win32::System::Pipes::SetNamedPipeHandleState;
#[cfg(windows)]
use windows_sys::Win32::System::Pipes::PIPE_NOWAIT;

#[cfg(windows)]
pub fn make_nonblocking_windows<H: AsRawHandle>(handle: H) -> std::io::Result<()> {
    use std::io;

    let raw = handle.as_raw_handle();
    let mut mode: u32 = PIPE_NOWAIT;

    let ok = unsafe { SetNamedPipeHandleState(raw as _, &mut mode, std::ptr::null_mut(), std::ptr::null_mut()) };

    if ok == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub struct Stream<R: AsFd + Read + Send + Debug + 'static> {
    reader: Arc<Mutex<Option<R>>>,
    handle: Option<std::thread::JoinHandle<()>>,
    rx: Receiver<u8>,
    shutdown_tx: Sender<()>,
}

impl<R: AsFd + Read + Send + Debug + 'static> Stream<R> {
    pub fn new(reader: R) -> Self {
        #[cfg(unix)]
        {
            make_nonblocking_unix(&reader).unwrap();
        }
        let (tx, rx) = bounded::<u8>(1024);
        let (shutdown_tx, shutdown_rx) = bounded::<()>(1);

        let reader = Arc::new(Mutex::new(Some(reader)));
        let reader_clone = Arc::clone(&reader);

        let handle = std::thread::spawn(move || {
            let mut buf = [0u8; 1];
            let mut reader = reader_clone.lock().unwrap().take().expect("wtf reader can't not be here");
            loop {
                crossbeam_channel::select! {
                    recv(shutdown_rx) -> _ => break,
                    default => {
                        if reader.read_exact(&mut buf).is_ok() && tx.send(buf[0]).is_err() {
                            break; // receiver dropped
                        }
                    }
                }
            }
            reader_clone.lock().unwrap().replace(reader);
        });

        Self { reader, rx, handle: Some(handle), shutdown_tx }
    }

    pub fn try_read(&self) -> Option<u8> {
        self.rx.try_recv().ok()
    }

    pub fn kill(&mut self, function_name: &'static str) -> LuaEmptyResult {
        if let Err(err) = self.shutdown_tx.send(()) {
            return wrap_err!("{}: unable to send shutdown signal to reader in other thread due to err: {}", function_name, err);
        }

        if let Some(handle) = self.handle.take() {
            // spawn yet another thread to avoid blocking main thread when reader.read_exact still blocks after thread's been joined
            std::thread::spawn(move || -> LuaEmptyResult {
                match handle.join() {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        wrap_err!("{}: unable to join reader thread due to err: {:?}", function_name, err)
                    }
                }
            });
        }

        Ok(())
    }

    pub fn take_reader(&mut self) -> Option<R> {
        self.reader.lock().unwrap().take()
    }

    pub fn recover(&mut self, function_name: &'static str, arc_stream: &Arc<Mutex<Option<R>>>) -> LuaEmptyResult {
        // dbg!(&self.reader);
        self.kill(function_name)?;

        let reader: R = {
            #[cfg(unix)]
            {
                let mut retries: usize = 0;
                loop {
                    if let Some(reader) = self.take_reader() {
                        break reader;
                    } else if retries < 6 {
                        retries += 1;
                        std::thread::sleep(std::time::Duration::from_millis(1))
                    } else {
                        return wrap_err!("{}: unable to recover reader from Stream", function_name)
                    }
                }
            }
        };
        *arc_stream.lock().unwrap() = Some(reader);
        Ok(())
    }
}