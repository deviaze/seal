
use mluau::prelude::*;
use crate::prelude::*;
use crossbeam_channel::{bounded, TrySendError, TryRecvError};

/// Wrapper around crossbeam channel's senders and receivers; actually contains our own
/// Sender/Receiver wrapper types that contain useful methods so we don't have to duplicate the same
/// logic in every send, readbytes, etc. etc. closure in mod.rs
pub struct Channel<T> {
    pub sender: Sender<T>,
    pub receiver: Receiver<T>,
}
impl<T> Channel<T> {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver): (crossbeam_channel::Sender<T>, crossbeam_channel::Receiver<T>) = bounded(capacity);
        Self { sender: Sender { sender }, receiver: Receiver { receiver } }
    }
}

/// we have to wrap crossbeam_channel::Sender in our own newtype
/// because of borrow checking rules (i originally wanted these methods on Channel instead)
pub struct Sender<T> {
    pub sender: crossbeam_channel::Sender<T>,
}
impl<T> Sender<T> {
    pub fn try_send(&self, data: T) -> Result<(), TrySendError<T>> {
        match self.sender.try_send(data) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
    pub fn send(&self, data: T, function_name: &'static str) -> LuaEmptyResult {
        match self.sender.send(data) {
            Ok(_) => Ok(()),
            Err(_) => {
                wrap_err!("{}: message couldn't be sent because the channel was unexpectedly disconnected", function_name)
            }
        }
    }
}
impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self { sender: self.sender.clone() }
    }
}

/// we have to wrap crossbeam_channel::Receiver in our own newtype
/// because of borrow checking rules (i originally wanted these methods on Channel instead)
pub struct Receiver<T> {
    receiver: crossbeam_channel::Receiver<T>
}
impl<T> Receiver<T> {
    pub fn try_recv(&self, function_name: &'static str) -> LuaResult<Option<T>> {
        match self.receiver.try_recv() {
            Ok(data) => Ok(Some(data)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => {
                wrap_err!("{} cannot receive on this channel because its receiver got disconnected", function_name)
            }
        }
    }
    pub fn recv_await(&self, function_name: &'static str) -> LuaResult<T> {
        match self.receiver.recv() {
            Ok(data) => Ok(data),
            Err(err) => {
                wrap_err!("{}: encountered a RecvError: {}", function_name, err)
            }
        }
    }
}
impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Self { receiver: self.receiver.clone() }
    }
}
