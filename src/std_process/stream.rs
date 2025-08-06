use crate::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::collections::VecDeque;

use mlua::prelude::*;

pub enum StreamType {
    Stdout,
    Stderr,
}

#[derive(Debug)]
pub enum TruncateSide {
    Front,
    Back,
}

/// Multithreaded wrapper type that abstracts reading from a child process' stdout or stderr.
///
/// This is a cross-platform compatible solution that makes sure reading from stdout/stderr is nonblocking;
/// instead of taking control of the stream's BufReader directly, manages an `inner` buffer (VecDeque).
///
/// A producer thread (called in `Stream::new()`) continually adds new data to `inner` as it comes in
/// from the stdout/stderr `io::Reader`. If adding the new data causes the buffer to exceed the requested capacity,
/// the producer thread truncates it back to the requested capacity.
/// When a reader method is called, only returns what's currently stored in the inner buffer.
///
/// This also allows us to customize the type of reader methods available because we're storing read content
/// before a read is requested by the end user; this allows `read_to` to work.
pub struct Stream {
    inner: Arc<Mutex<VecDeque<u8>>>,
    join_handle: Option<JoinHandle<Result<(), LuaError>>>,
    stream_type: StreamType,
    still_reading: Arc<AtomicBool>,
    capacity: usize,
}

impl Stream {
    /// Spins up the producer child thread, saving its handle and returning the new `Stream`.
    pub fn new<R: Read + Send + 'static>(function_name: &'static str, mut reader: R, stream_type: StreamType, capacity: usize, truncate_side: TruncateSide) -> LuaResult<Self> {
        let inner = Arc::new(Mutex::new(VecDeque::<u8>::with_capacity(capacity)));
        let inner_clone = Arc::clone(&inner);

        let still_reading = Arc::new(AtomicBool::new(true));
        let still_reading_clone = Arc::clone(&still_reading);

        let arc_capacity = Arc::new(capacity);

        let handle = std::thread::spawn(move || -> Result<(), LuaError> {
            let mut buffer = [0u8; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(bytes_read) => {
                        // let mut inner = inner_clone.lock().unwrap();
                        let mut inner = match inner_clone.lock() {
                            Ok(l) => l,
                            Err(err) => {
                                return wrap_err!("reader thread unable to lock because it got poisoned: {}", err);
                            }
                        };
                        inner.extend(buffer[..bytes_read].iter());
                        if inner.len() >= *arc_capacity {
                            let extra_byte_count = inner.len().saturating_sub(*arc_capacity);
                            match truncate_side {
                                TruncateSide::Front => {
                                    let bytes_to_remove = inner.drain(..extra_byte_count); // keep these ranges non-inclusive to prevent off-by-one issues
                                    drop(bytes_to_remove);
                                },
                                TruncateSide::Back => {
                                    for _ in 0..extra_byte_count { // why dont vecdeques have drain_back hmm?
                                        if inner.pop_back().is_none() {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Err(err) => {
                        return wrap_err!("{}: error reading process stdout/stderr into Stream buffer: {}", function_name, err);
                    }
                }
            }
            // flip still_reading here
            still_reading_clone.store(false, Ordering::Relaxed);
            Ok(())
        });

        Ok(Self {
            inner,
            join_handle: Some(handle),
            stream_type,
            still_reading,
            capacity,
        })
    }

    /// Checks the status of the running thread; if the thread has died, joins it and propagates errors
    /// from that thread upwards.
    fn alive(&mut self, function_name: &'static str) -> LuaEmptyResult {
        if let Some(handle) = self.join_handle.take_if(|h| h.is_finished()) {
            match handle.join() {
                Ok(Ok(_)) => {
                    Ok(())
                    // match self.inner.try_lock() {
                    //     Ok(_) => {
                    //         Ok(())
                    //         // getting a bunch of "called on a dead child" after literal child:alive() checks in a loop
                    //         // is bad ux, not sure how to handle this
                    //         // if inner.is_empty() {
                    //         //     wrap_err!("{} called on a dead child with an empty stream", function_name)
                    //         // } else {
                    //         //     Ok(()) // allow reading from stream if stream isn't empty
                    //         // }
                    //     },
                    //     Err(_err) => {
                    //         wrap_err!("{} called on a dead child", function_name)
                    //     }
                    // }
                },
                Ok(Err(err)) => Err(err),
                Err(err) => {
                    wrap_err!("{}: Unexpected error checking whether the thread reading from stdout/stderr is dead or alive: {:#?}", function_name, err)
                }
            }
        } else {
            Ok(())
        }
    }

    fn pop_timeout(&self, mut multivalue: LuaMultiValue, function_name: &'static str) -> LuaResult<Option<Duration>> {
        let f = match multivalue.pop_front() {
            Some(LuaValue::Number(f)) => f,
            Some(LuaValue::Integer(i)) => i as f64,
            Some(LuaNil) | None => {
                return Ok(None);
            },
            Some(other) => {
                return wrap_err!("{} expected timeout to be a number or nil, got: {:?}", function_name, other);
            }
        };

        if f.is_nan() || f.is_infinite() {
            wrap_err!("{}: timeout can't be NaN nor infinite!", function_name)
        } else if f < 0.0 {
            wrap_err!("{}: timeout can't be negative! got: {:?}", function_name, f)
        } else {
            let duration = match Duration::try_from_secs_f64(f) {
                Ok(duration) => duration,
                Err(err) => {
                    return wrap_err!("{}: error creating Duration from timeout: {}", function_name, err);
                }
            };
            Ok(Some(duration))
        }
    }

    pub fn read_exact(&mut self, luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
        let function_name = match self.stream_type {
            StreamType::Stdout => "ChildProcess.stdout:read_exact(count: number, timeout: number?)",
            StreamType::Stderr => "ChildProcess.stderr:read_exact(count: number, timeout: number?)",
        };
        self.alive(function_name)?;
        pop_self(&mut multivalue, function_name)?;

        let count = match multivalue.pop_front() {
            Some(LuaValue::Integer(i)) => int_to_usize(i, function_name, "count")?,
            Some(LuaValue::Number(f)) => float_to_usize(f, function_name, "count")?,
            None => {
                return wrap_err!("{} called without required argument 'count'");
            },
            Some(other) => {
                return wrap_err!("{} expected count to be an integer number, got: {:?}", function_name, other);
            }
        };

        if count == 0 {
            return wrap_err!("{}: why do you want to read 0 bytes from the stream???", function_name);
        }

        let timeout = self.pop_timeout(multivalue, function_name)?;
        let start_time = if timeout.is_some() {
            Some(Instant::now())
        } else {
            None
        };

        loop {
            let mut inner = self.inner.lock().unwrap();
            if inner.len() <= count {
                if !&self.still_reading.load(Ordering::Relaxed) {
                    return Ok(LuaNil);
                } else if let Some(timeout) = timeout && timeout.is_zero() { // user passed 0.0 duration for nonblocking behavior
                    return Ok(LuaNil)
                } else if let Some(timeout) = timeout
                    && let Some(start_time) = start_time
                    && start_time.elapsed() >= timeout    
                {
                    return Ok(LuaNil);
                } else {
                    // explicitly drop mutex to unlock inner
                    drop(inner);
                    // allow some time for reader thread to add stuff to inner before continuing to next iteration
                    std::thread::sleep(Duration::from_millis(10));
                }
            } else {
                let bytes_read: Vec<u8> = inner.drain(..count).collect();
                return ok_string(bytes_read, luau);
            }
        }
    }

    pub fn read(&mut self, luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
        let function_name = match self.stream_type {
            StreamType::Stdout => "ChildProcess.stdout:read(count: number?, timeout: number?)",
            StreamType::Stderr => "ChildProcess.stderr:read(count: number?, timeout: number?)",
        };
        self.alive(function_name)?;
        pop_self(&mut multivalue, function_name)?;

        let count = match multivalue.pop_front() {
            Some(LuaValue::Integer(i)) => int_to_usize(i, function_name, "count")?,
            Some(LuaValue::Number(f)) => float_to_usize(f, function_name, "count")?,
            Some(LuaNil) | None => self.capacity,
            Some(other) => {
                return wrap_err!("{} expected count to be an integer number, got: {:?}", function_name, other);
            }
        };

        if count == 0 {
            return wrap_err!("{}: why do you want to read 0 bytes from the stream???", function_name);
        }

        let timeout = self.pop_timeout(multivalue, function_name)?;
        let start_time = if timeout.is_some() {
            Some(Instant::now())
        } else {
            None
        };

        loop {
            let mut inner = match self.inner.lock() {
                Ok(l) => l,
                Err(err) => {
                    return wrap_err!("{}: unable to lock inner because poisoned: {}", function_name, err);
                }
            };
            if inner.is_empty() {
                if !&self.still_reading.load(Ordering::Relaxed) {
                    return Ok(LuaNil);
                } else if let Some(timeout) = timeout && timeout.is_zero() { // user passed 0.0 duration for nonblocking behavior
                    return Ok(LuaNil)
                } else if let Some(timeout) = timeout
                    && let Some(start_time) = start_time
                    && start_time.elapsed() >= timeout    
                {
                    return Ok(LuaNil);
                } else {
                    // explicitly drop mutex to unlock inner
                    drop(inner);
                    // allow some time for reader thread to add stuff to inner before continuing to next iteration
                    std::thread::sleep(Duration::from_millis(10));
                }
            } else {
                let count = std::cmp::min(inner.len(), count);
                let bytes_read: Vec<u8> = inner.drain(..count).collect();
                return ok_string(bytes_read, luau);
            }
        }
    }

    pub fn read_to(&mut self, luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
        let function_name = match self.stream_type {
            StreamType::Stdout => "ChildProcess.stdout:read_to(term: string, inclusive: boolean?, timeout: number?, allow_partial: boolean?)",
            StreamType::Stderr => "ChildProcess.stderr:read_to(term: string, inclusive: boolean?, timeout: number?, allow_partial: boolean?)",
        };
        self.alive(function_name)?;
        pop_self(&mut multivalue, function_name)?;

        let search_term = match multivalue.pop_front() {
            Some(LuaValue::String(t)) => t.as_bytes().to_vec(),
            Some(LuaNil) => {
                return wrap_err!("{} expected search term to be a string, got nil", function_name);
            },
            Some(other) => {
                return wrap_err!("{} expected search term to be a string, got: {:?}", function_name, other);
            },
            None => {
                return wrap_err!("{} expected search term (string), but was incorrectly called with zero arguments", function_name);
            }
        };

        let inclusive = match multivalue.pop_front() {
            Some(LuaValue::Boolean(inclusive)) => inclusive,
            Some(LuaNil) | None => false,
            Some(other) => {
                return wrap_err!("{} expected inclusive to be a boolean or nil (default false), got: {:?}", function_name, other);
            }
        };

        let timeout = match multivalue.pop_front() {
            Some(LuaValue::Integer(i)) => Some(Duration::from_secs_f64(i as f64)),
            Some(LuaValue::Number(f)) => Some(Duration::from_secs_f64(f)),
            Some(LuaNil) | None => None,
            Some(other) => {
                return wrap_err!("{} expected timeout to be a number (in seconds) or nil, got: {:?}", function_name, other);
            }
        };

        let start_time = if timeout.is_some() {
            Some(Instant::now())
        } else {
            None
        };

        let allow_partial = match multivalue.pop_front() {
            Some(LuaValue::Boolean(partial)) => partial,
            Some(LuaNil) | None => false,
            Some(other) => {
                return wrap_err!("{} expected allow_partial to be a boolean or nil (default false), got: {:?}", function_name, other);
            }
        };

        let still_reading = Arc::clone(&self.still_reading);

        loop {
            // this is ugly asf we don't want to lock inner if we don't need to
            let should_return_if_allow_partial = if
                let Some(start_time) = start_time
                && let Some(timeout) = timeout
                && start_time.elapsed() >= timeout
            {
                if allow_partial {
                    true
                } else {
                    return Ok(LuaNil);
                }
            } else {
                false
            };

            let mut inner = self.inner.lock().unwrap();
            if should_return_if_allow_partial {
                let drained: Vec<u8> = inner.drain(..).collect();
                return ok_string(&drained, luau);
            }
            if inner.is_empty() || inner.len() < search_term.len() {
                drop(inner);
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }

            if search_term.is_empty() {
                // why would someone want to search for an empty string?
                // we already have a way to consume 1 character at a time, so I guess this means
                // we should read to end!
                if !still_reading.load(Ordering::Relaxed) {
                    let drained: Vec<u8> = inner.drain(..).collect();
                    return ok_string(&drained, luau);
                }
            } else if search_term.len() == 1 {
                // we can optimize by just doing iter().position for single char search strings
                if let Some(pos) = inner.iter().position(|u| &search_term[0] == u) {
                    let mut drained: Vec<u8> = inner.drain(..=pos).collect();
                    if !inclusive && !drained.is_empty() {
                        drained.pop();
                    }
                    return ok_string(&drained, luau)
                }
            } else {
                // using a sliding window algorithm to look across the entire stream
                // without having to allocate more than just the search term in terms of length

                // we have to fix the internal representation of the VecDeque so that .as_slices
                // behaves as expected for windowing (returning front, back where front has the whole contents)
                // instead of the middle point being unspecified
                inner.make_contiguous();

                let slice = inner.as_slices().0;
                let mut search_position: Option<usize> = None;
                let mut window: VecDeque<u8> = VecDeque::with_capacity(search_term.len());

                for (pos, byte) in slice.iter().enumerate() {
                    if window.len() == search_term.len() {
                        // shift window to the left, this can be O(n) but we error on the side
                        // of users passing in small search terms and hope it's okay
                        window.pop_front();
                    }
                    window.push_back(*byte);
                    // TODO: use vec.push_within_capacity when stabilized
                    // match window.push_within_capacity(byte) {
                    //     Ok(_) => {},
                    //     Err(byte) => {
                    //         panic!("{}: pushing a byte ({}) into window is not supposed to allocate additional capacity", function_name, byte);
                    //     }
                    // }
                    // if window == search_term {
                    //     search_position = Some(pos + 1); // +1 makes it inclusive
                    //     break;
                    // }
                    if window.len() == search_term.len()
                        // apparently doing an iter with eq is supposedly faster and doesn't allocate
                        // than comparing window (Vec<u8>) == search_term (VecDeque<u8>)
                        && window.iter().eq(search_term.iter())
                        // if there's issues, use this instead:
                        // && window.iter().copied().eq(search_term.iter().copied())
                    {
                        search_position = Some(pos + 1); // +1 makes it inclusive
                        break;
                    }
                }

                if let Some(found_pos) = search_position {
                    let mut drained: Vec<u8> = inner.drain(..found_pos).collect();
                    if !inclusive {
                        drained.truncate(found_pos - search_term.len());
                    }
                    return ok_string(&drained, luau);
                }
            }
        }
    }

    pub fn fill(&mut self, _luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
        let function_name = match self.stream_type {
            StreamType::Stdout => "ChildProcess.stdout:fill(target: buffer, target_offset: number?, timeout: number?)",
            StreamType::Stderr => "ChildProcess.stderr:fill(target: buffer, target_offset: number?, timeout: number?)",
        };
        self.alive(function_name)?;
        pop_self(&mut multivalue, function_name)?;

        let buffy = match multivalue.pop_front() {
            Some(LuaValue::Buffer(buffy)) => buffy,
            Some(other) => {
                return wrap_err!("{} expected target to be a buffer, got: {:?}", function_name, other);
            },
            None => {
                return wrap_err!("{} incorrectly called without target buffer", function_name);
            }
        };

        let target_offset = match multivalue.pop_front() {
            Some(LuaValue::Integer(offset)) => int_to_usize(offset, function_name, "target_offset")?,
            Some(LuaValue::Number(f)) => float_to_usize(f, function_name, "target_offset")?,
            Some(LuaNil) | None => 0,
            Some(other) => {
                return wrap_err!("{} expected target_offset to be a number or nil, got: {:?}", function_name, other);
            }
        };

        if target_offset > buffy.len() - 1 {
            return wrap_err!("{}: target_offset {} > buffer_length - 1 {} (buffer would overflow); add an explicit target_offset < buffer_length - 1 check", function_name, target_offset, buffy.len());
        }

        let timeout = self.pop_timeout(multivalue, function_name)?;
        let start_time = if timeout.is_some() {
            Some(Instant::now())
        } else {
            None
        };

        loop {
            let mut inner = self.inner.lock().unwrap();
            if inner.is_empty() {
                if !&self.still_reading.load(Ordering::Relaxed) {
                    return Ok(LuaValue::Integer(0));
                } else if let Some(timeout) = timeout && timeout.is_zero() { // user passed 0.0 duration for nonblocking behavior
                    return Ok(LuaValue::Integer(0));
                } else if let Some(timeout) = timeout
                    && let Some(start_time) = start_time
                    && start_time.elapsed() >= timeout    
                {
                    return Ok(LuaValue::Integer(0));
                } else {
                    // explicitly drop mutex to unlock inner
                    drop(inner);
                    // allow some time for reader thread to add stuff to inner before continuing to next iteration
                    std::thread::sleep(Duration::from_millis(10));
                }
            } else {
                let last_index = {
                    // we only want to drain as many bytes can fit into buffy since users can't specify how many bytes are expected up front
                    // (use stream:readbytes_exact for that usecase instead)
                    let space_left = buffy.len().saturating_sub(target_offset);
                    // similarly, we don't want to try to read more bytes in inner than actually exist (causes a panic), so we must clamp to inner's length
                    let max_index_in_inner = inner.len().saturating_sub(1);
                    std::cmp::min(max_index_in_inner, space_left.saturating_sub(1))
                };
                let bytes_read: Vec<u8> = inner.drain(..=last_index).collect();
                if bytes_read.is_empty() {
                    return Ok(LuaValue::Integer(0))
                } else if target_offset + bytes_read.len() <= buffy.len() { // should've already been checked by precondition above but why not check again in case smth changed
                    buffy.write_bytes(target_offset, &bytes_read);
                    let byte_count: i64 = match bytes_read.len().try_into() {
                        Ok(i) => i,
                        Err(_) => {
                            return wrap_err!("{}: cannot convert the number of bytes read (usize) into i64");
                        }
                    };
                    return Ok(LuaValue::Integer(byte_count))
                } else {
                    unreachable!(
                        "{}: logic bug. drained more bytes than buffer space allowed (offset {} + drained {}) into buffer of size {}",
                        function_name, target_offset, bytes_read.len(), buffy.len()
                    )
                }
            }
        }
    }

    pub fn fill_exact(&mut self, _luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
        let function_name = match self.stream_type {
            StreamType::Stdout => "ChildProcess.stdout:fill_exact(count: number, target: buffer, offset: number?, timeout: number?)",
            StreamType::Stderr => "ChildProcess.stderr:fill_exact(count: number, target: buffer, offset: number?, timeout: number?)",
        };
        self.alive(function_name)?;
        pop_self(&mut multivalue, function_name)?;

        let count = match multivalue.pop_front() {
            Some(LuaValue::Integer(i)) => int_to_usize(i, function_name, "count")?,
            Some(LuaValue::Number(f)) => float_to_usize(f, function_name, "count")?,
            Some(LuaNil) | None => {
                return wrap_err!("{} expected count to be a number, got nothing or nil", function_name);
            },
            Some(other) => {
                return wrap_err!("{} expected count to be a number, got: {:?}", function_name, other);
            }
        };

        let buffy = match multivalue.pop_front() {
            Some(LuaValue::Buffer(buffy)) => buffy,
            Some(LuaNil) | None => {
                return wrap_err!("{} expected target to be a buffer, got nothing or nil", function_name);
            },
            Some(other) => {
                return wrap_err!("{} expected target to be a buffer, got: {:?}", function_name, other);
            }
        };

        let target_offset = match multivalue.pop_front() {
            Some(LuaValue::Integer(offset)) => int_to_usize(offset, function_name, "offset")?,
            Some(LuaValue::Number(f)) => float_to_usize(f, function_name, "offset")?,
            Some(LuaNil) | None => 0,
            Some(other) => {
                return wrap_err!("{} expected offset to be a number or nil, got: {:?}", function_name, other);
            }
        };

        let timeout = self.pop_timeout(multivalue, function_name)?;
        let start_time = if timeout.is_some() {
            Some(Instant::now())
        } else {
            None
        };

        loop {
            let mut inner = self.inner.lock().unwrap();
            if inner.is_empty() || inner.len() - 1 < count {
                if !&self.still_reading.load(Ordering::Relaxed) {
                    return Ok(LuaValue::Boolean(false));
                } else if let Some(timeout) = timeout && timeout.is_zero() { // user passed 0.0 duration for nonblocking behavior
                    return Ok(LuaValue::Boolean(false));
                } else if let Some(timeout) = timeout
                    && let Some(start_time) = start_time
                    && start_time.elapsed() >= timeout    
                {
                    return Ok(LuaValue::Boolean(false));
                } else {
                    // explicitly drop mutex to unlock inner
                    drop(inner);
                    // allow some time for reader thread to add stuff to inner before continuing to next iteration
                    std::thread::sleep(Duration::from_millis(10));
                }
            } else if target_offset + count <= buffy.len() {
                let bytes_read: Vec<u8> = inner.drain(..count).collect();
                buffy.write_bytes(target_offset, &bytes_read);
                return Ok(LuaValue::Boolean(true));
            } else {
                return wrap_err!("{}: can't fit offset {} + count {} bytes into buffer of length {}", function_name, target_offset, count, buffy.len());
            }
        }

    }

    /// iterate over the lines of inner, only consuming bytes when \n is reached
    pub fn lines(&mut self, luau: &Lua, mut multivalue: LuaMultiValue) -> LuaResult<LuaFunction> {
        let function_name = match self.stream_type {
            StreamType::Stdout => "ChildProcess.stdout:lines()",
            StreamType::Stderr => "ChildProcess.stderr:lines()",
        };
        self.alive(function_name)?;
        pop_self(&mut multivalue, function_name)?;

        let timeout = {
            let timeout = match multivalue.pop_front() {
                Some(LuaValue::Integer(i)) => Some(i as f64),
                Some(LuaValue::Number(f)) => Some(f),
                Some(LuaNil) | None => None,
                Some(other) => {
                    return wrap_err!("{} expected timeout to be a number or nil, got: {:?}", function_name, other);
                }
            };

            if let Some(timeout) = timeout && timeout.is_nan() {
                return wrap_err!("{}: timeout is unexpectedly NaN 💀", function_name)
            } else if let Some(timeout) = timeout && timeout.is_sign_negative() {
                return wrap_err!("{}: timeout should be positive (got: {})", function_name, timeout);
            } else {
                timeout
            }
        };

        let timeout_start_time = if timeout.is_some() {
            Some(Instant::now())
        } else {
            None
        };

        let inner = Arc::clone(&self.inner);
        let still_reading = Arc::clone(&self.still_reading);
        luau.create_function_mut({
            move | luau: &Lua, _value: LuaValue | -> LuaValueResult {
                let function_name = "ChildProcess.stream:lines() iterator function";
                loop { // we keep loopin because if we didn't find \n we can't return nil yet or it'd stop iteration
                    let mut inner = match inner.lock() {
                        Ok(inner) => inner,
                        Err(err) => {
                            return wrap_err!("{}: unable to lock inner due to err: {}", function_name, err);
                        }
                    };

                    if let Some(position) = inner.iter().position(|&b| b == b'\n') {
                        // since we've found a \n, we're free to drain inner and consume those bytes off the stream
                        let trimmed_bytes = {
                            let bytes_with_newline: Vec<u8> = inner.drain(..=position).collect();
                            // trim possible \r prefix without getting rid of possibly wanted whitespace
                            let start_pos: usize = if bytes_with_newline.first() == Some(&b'\r') { 1 } else { 0 };
                            // users don't want a \n if they're iterating line by line
                            let end_pos: usize = bytes_with_newline.len() - 1;
                            bytes_with_newline[start_pos..end_pos].to_vec()
                        };
                        return ok_string(&trimmed_bytes, luau)
                    } else if !still_reading.load(Ordering::Relaxed) {
                        return Ok(LuaNil)
                    } else {
                        if let Some(start_time) = timeout_start_time && let Some(timeout) = timeout {
                            let elapsed = start_time.elapsed();
                            if elapsed.as_secs_f64() >= timeout {
                                return Ok(LuaNil)
                            }
                        }
                        // manually release mutex lock on inner so writer thread can add more bytes while we wait
                        drop(inner);
                        // allow for some time for writer to add bytes before continuing to next iteration
                        std::thread::sleep(Duration::from_millis(10));
                    }
                }
            }
        })
    }

    pub fn iter(&mut self, luau: &Lua, mut multivalue: LuaMultiValue) -> LuaResult<LuaFunction> {
        let function_name = match self.stream_type {
            StreamType::Stdout => "ChildProcess.stdout:__iter()",
            StreamType::Stderr => "ChildProcess.stderr:__iter()",
        };
        self.alive(function_name)?;
        pop_self(&mut multivalue, function_name)?;

        let timeout = {
            let timeout = match multivalue.pop_front() {
                Some(LuaValue::Integer(i)) => Some(i as f64),
                Some(LuaValue::Number(f)) => Some(f),
                Some(LuaNil) | None => None,
                Some(other) => {
                    return wrap_err!("{} expected timeout to be a number or nil, got: {:?}", function_name, other);
                }
            };

            if let Some(timeout) = timeout && timeout.is_nan() {
                return wrap_err!("{}: timeout is unexpectedly NaN 💀", function_name)
            } else if let Some(timeout) = timeout && timeout.is_sign_negative() {
                return wrap_err!("{}: timeout should be positive (got: {})", function_name, timeout);
            } else {
                timeout
            }
        };

        let timeout_start_time = if timeout.is_some() {
            Some(Instant::now())
        } else {
            None
        };

        let write_delay_ms = Duration::from_millis(
            match multivalue.pop_front() {
                Some(LuaValue::Integer(i)) => int_to_u64(i, function_name, "write_delay_ms")?,
                Some(LuaValue::Number(f)) => float_to_u64(f, function_name, "write_delay_ms")?,
                Some(LuaNil) | None => 5_u64,
                Some(other) => {
                    return wrap_err!("{} expected write_delay_ms to be a number (convertible to u64), got: {:?}", function_name, other);
                }
            }
        );

        let inner = Arc::clone(&self.inner);
        let still_reading = Arc::clone(&self.still_reading);
        luau.create_function_mut({
            move | luau: &Lua, _value: LuaValue | -> LuaValueResult {
                let function_name = "ChildProcess.stream iterator function";
                loop {
                    let mut inner = match inner.lock() {
                        Ok(inner) => inner,
                        Err(err) => {
                            return wrap_err!("{}: unable to lock inner due to err: {}", function_name, err);
                        }
                    };

                    if !inner.is_empty() {
                        let bytes_read: Vec<u8> = inner.drain(..).collect();
                        return ok_string(&bytes_read, luau)
                    } else if !still_reading.load(Ordering::Relaxed) {
                        return Ok(LuaNil)
                    } else {
                        if let Some(start_time) = timeout_start_time && let Some(timeout) = timeout {
                            let elapsed = start_time.elapsed();
                            if elapsed.as_secs_f64() >= timeout {
                                return Ok(LuaNil)
                            }
                        }
                        // manually release mutex lock on inner so writer thread can add more bytes while we wait
                        drop(inner);
                        // allow for some time for writer to add bytes before continuing to next iteration
                        std::thread::sleep(write_delay_ms);
                    }
                }
            }
        })
    }

    pub fn len(&mut self) -> LuaResult<usize> {
        let function_name = match self.stream_type {
            StreamType::Stdout => "ChildProcess.stdout:len()",
            StreamType::Stderr => "ChildProcess.stderr:len()",
        };
        let inner = match self.inner.lock() {
            Ok(inner) => inner,
            Err(err) => {
                return wrap_err!("{}: unable to lock inner due to err: {}", function_name, err);
            }
        };
        Ok(inner.len())
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn create_handle(stream_cell: Rc<RefCell<Self>>, luau: &Lua) -> LuaResult<LuaTable> {
        TableBuilder::create(luau)?
            .with_function("read_exact", {
                let stream_cell = Rc::clone(&stream_cell);
                move | luau: &Lua, multivalue: LuaMultiValue | -> LuaValueResult {
                    let function_name = "ChildProcessStream:read_exact(count: number, timeout: number?)";
                    match stream_cell.try_borrow_mut() {
                        Ok(ref mut stream) => stream.read_exact(luau, multivalue),
                        Err(_) => wrap_err!("{}: stream already borrowed", function_name)
                    }
                }
            })?
            .with_function("read", {
                let stream_cell = Rc::clone(&stream_cell);
                move | luau: &Lua, multivalue: LuaMultiValue | -> LuaValueResult {
                    let function_name = "ChildProcessStream:read(count: number?, timeout: number?)";
                    match stream_cell.try_borrow_mut() {
                        Ok(ref mut stream) => stream.read(luau, multivalue),
                        Err(_) => wrap_err!("{}: stream already borrowed", function_name)
                    }
                }
            })?
            .with_function("read_to", {
                let stream_cell = Rc::clone(&stream_cell);
                move | luau: &Lua, multivalue: LuaMultiValue | -> LuaValueResult {
                    let function_name = "ChildProcessStream:read_to(term: string, inclusive: boolean?, timeout: number?)";
                    match stream_cell.try_borrow_mut() {
                        Ok(ref mut stream) => stream.read_to(luau, multivalue),
                        Err(_) => wrap_err!("{}: stream already borrowed", function_name)
                    }
                }
            })?
            .with_function("fill", {
                let stream_cell = Rc::clone(&stream_cell);
                move | luau: &Lua, multivalue: LuaMultiValue | -> LuaValueResult {
                    let function_name = "ChildProcessStream:fill(target: buffer, target_offset: number?, timeout: number?)";
                    match stream_cell.try_borrow_mut() {
                        Ok(ref mut stream) => stream.fill(luau, multivalue),
                        Err(_) => wrap_err!("{}: stream already borrowed", function_name)
                    }
                }
            })?
            .with_function("fill_exact", {
                let stream_cell = Rc::clone(&stream_cell);
                move | luau: &Lua, multivalue: LuaMultiValue | -> LuaValueResult {
                    let function_name = "ChildProcessStream:fill_exact(count: number, target: buffer, target_offset: number?, timeout: number?)";
                    match stream_cell.try_borrow_mut() {
                        Ok(ref mut stream) => stream.fill_exact(luau, multivalue),
                        Err(_) => wrap_err!("{}: stream already borrowed", function_name)
                    }
                }
            })?
            .with_function("lines", {
                let stream_cell = Rc::clone(&stream_cell);
                move | luau: &Lua, multivalue: LuaMultiValue | -> LuaResult<LuaFunction> {
                    let function_name = "ChildProcessStream:lines()";
                    match stream_cell.try_borrow_mut() {
                        Ok(ref mut stream) => stream.lines(luau, multivalue),
                        Err(_) => wrap_err!("{}: stream already borrowed", function_name),
                    }
                }
            })?
            .with_function("len", {
                let stream_cell = Rc::clone(&stream_cell);
                move | _luau: &Lua, _value: LuaValue | -> LuaValueResult {
                    let function_name = "ChildProcessStream:len()";
                    match stream_cell.try_borrow_mut() {
                        Ok(ref mut stream) => {
                            Ok(LuaValue::Integer(stream.len()? as i64))
                        },
                        Err(_) => wrap_err!("{}: stream already borrowed", function_name),
                    }
                }
            })?
            .with_function("capacity", {
                let stream_cell = Rc::clone(&stream_cell);
                move | _luau: &Lua, _value: LuaValue | -> LuaValueResult {
                    let function_name = "ChildProcessStream:capacity()";
                    match stream_cell.try_borrow_mut() {
                        Ok(ref mut stream) => {
                            Ok(LuaValue::Integer(stream.capacity() as i64))
                        },
                        Err(_) => wrap_err!("{}: stream already borrowed", function_name),
                    }
                }
            })?
            // users can't iterate and supply a timeout with generalized iteration
            .with_function("iter", {
                let stream_cell = Rc::clone(&stream_cell);
                move | luau: &Lua, multivalue: LuaMultiValue | -> LuaResult<LuaFunction> {
                    let function_name = "ChildProcessStream:iter(timeout: number?, write_delay_ms: number?)";
                    match stream_cell.try_borrow_mut() {
                        Ok(ref mut stream) => stream.iter(luau, multivalue),
                        Err(_) => wrap_err!("{}: stream already borrowed", function_name),
                    }
                }
            })?
            .with_metatable(TableBuilder::create(luau)?
                .with_function("__iter", {
                    let stream_cell = Rc::clone(&stream_cell);
                    move | luau: &Lua, multivalue: LuaMultiValue | -> LuaResult<LuaFunction> {
                        let function_name = "ChildProcessStream:__iter()";
                        match stream_cell.try_borrow_mut() {
                            Ok(ref mut stream) => stream.iter(luau, multivalue),
                            Err(_) => wrap_err!("{}: stream already borrowed", function_name),
                        }
                    }
                })?
                .build_readonly()?
            )?
            .build_readonly()
    }
}
