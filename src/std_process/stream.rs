use crate::prelude::*;
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

pub struct Stream {
    inner: Arc<Mutex<VecDeque<u8>>>,
    join_handle: Option<JoinHandle<Result<(), LuaError>>>,
    stream_type: StreamType,
    still_reading: Arc<AtomicBool>,
    capacity: usize,
    start_instant: Instant,
}

impl Stream {
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
                        let mut inner = inner_clone.lock().unwrap();
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
            start_instant: Instant::now(),
        })
    }

    /// Checks the status of the running thread; if the thread has died, joins it and propagates errors
    /// from that thread upwards.
    fn alive(&mut self, function_name: &'static str) -> LuaEmptyResult {
        if let Some(handle) = self.join_handle.take_if(|h| h.is_finished()) {
            match handle.join() {
                Ok(Ok(_)) => {
                    match self.inner.try_lock() {
                        Ok(inner) => {
                            if inner.is_empty() {
                                wrap_err!("{} called on a dead child with an empty stream", function_name)
                            } else {
                                Ok(()) // allow reading from stream if stream isn't empty
                            }
                        },
                        Err(_err) => {
                            wrap_err!("{} called on a dead child", function_name)
                        }
                    }
                },
                Ok(Err(err)) => Err(err),
                Err(err) => {
                    wrap_err!("Unexpected error checking whether the thread reading from stdout/stderr is dead or alive: {:#?}", err)
                }
            }
        } else {
            Ok(())
        }
    }

    fn check_valid_duration(f: f64, function_name: &'static str) -> LuaResult<f64> {
        if f.is_nan() || f.is_infinite() {
            wrap_err!("{}: duration can't be NaN nor infinite", function_name)
        } else if f < 0.0 {
            wrap_err!("{}: duration can't be negative", function_name)
        } else {
            Ok(f)
        }
    }

    /// blocks for the user specified duration, or if unspecified,
    /// if the ChildProcess has literally just started, blocks for 1 ms, allowing for time for the writer thread
    /// to write something to the buffer.
    /// 
    /// if the user explicitly passes 0.0 seconds, we don't want to block, even if the `ChildProcess` has just started
    fn sleep_if_needed(&mut self, duration: Option<f64>) {
        if let Some(sleep_time) = duration && sleep_time != 0.0 {
            let sleep_duration = Duration::from_secs_f64(sleep_time);
            std::thread::sleep(sleep_duration);
        } else if let Some(sleep_time) = duration && sleep_time == 0.0 {
            // user explicitly doesn't want to block, don't block
        } else if duration.is_none() && self.start_instant.elapsed() < Duration::from_millis(1) {
            // if the child process has literally just started and the user hasn't specified a duration,
            // wait 1 ms before reading because it's very likely the child process hasn't written anything yet
            // and making the user explicitly forced to pass in a duration might be bad ux
            std::thread::sleep(Duration::from_millis(10));
        };
    }

    /// Reads exactly `count` bytes from the inner buffer, draining and returning those bytes as a string if count <= inner.len()
    /// else returns `nil`
    pub fn read_exact(&mut self, luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
        let function_name = match self.stream_type {
            StreamType::Stdout => "ChildProcess.stdout:read_exact(count: number, duration: number?)",
            StreamType::Stderr => "ChildProcess.stderr:read_exact(count: number, duration: number?)",
        };
        self.alive(function_name)?;
        pop_self(&mut multivalue, function_name)?;
        
        let count = match multivalue.pop_front() {
            Some(LuaValue::Integer(i)) => int_to_usize(i, function_name, "count")?,
            Some(LuaValue::Number(f)) => float_to_usize(f, function_name, "count")?,
            None => {
                return wrap_err!("{} expected count (integer number) but was called with zero arguments (not even nil)");
            },
            Some(other) => {
                return wrap_err!("{} expected count to be an integer number, got: {:?}", function_name, other);
            }
        };

        let duration = match multivalue.pop_front() {
            Some(LuaValue::Number(f)) => Some(Self::check_valid_duration(f, function_name)?),
            Some(LuaValue::Integer(i)) => {
                let f = i as f64;
                Some(Self::check_valid_duration(f, function_name)?)
            },
            Some(LuaNil) | None => None,
            Some(other) => {
                return wrap_err!("{} expected duration to be a number, got: {:?}", function_name, other);
            }
        };

        self.sleep_if_needed(duration);

        let mut inner = self.inner.lock().unwrap();
        if count <= inner.len() {
            let bytes_read: Vec<u8> = inner.drain(..count).collect();
            ok_string(bytes_read, luau)
        } else {
            // we can't read exactly buffer_size bytes from inner, so don't take anything and return nil
            Ok(LuaNil)
        }
    }

    /// Tries to read everything in the inner buffer, optionally blocking for `duration` before reading if specified
    pub fn read(&mut self, luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
        let function_name = match self.stream_type {
            StreamType::Stdout => "ChildProcess.stdout:read(duration: number?)",
            StreamType::Stderr => "ChildProcess.stderr:read(duration: number?)",
        };
        self.alive(function_name)?;
        pop_self(&mut multivalue, function_name)?;

        let duration = match multivalue.pop_front() {
            Some(LuaValue::Number(f)) => Some(Self::check_valid_duration(f, function_name)?),
            Some(LuaValue::Integer(i)) => {
                let f = i as f64;
                Some(Self::check_valid_duration(f, function_name)?)
            },
            Some(LuaNil) => Some(0.0), // user explicitly passed nil, they don't want to block
            None => None,
            Some(other) => {
                return wrap_err!("{} expected duration to be a number, got: {:?}", function_name, other);
            }
        };

        self.sleep_if_needed(duration);

        let mut inner = self.inner.lock().unwrap();
        if inner.is_empty() {
            Ok(LuaNil)
        } else {
            let bytes_read: Vec<u8> = inner.drain(..).collect();
            ok_string(bytes_read, luau)
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

    pub fn readbytes(&mut self, _luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
        let function_name = match self.stream_type {
            StreamType::Stdout => "ChildProcess.stdout:readbytes(target: buffer, offset: number?, duration: number?)",
            StreamType::Stderr => "ChildProcess.stderr:readbytes(target: buffer, offset: number?, duration: number?)",
        };
        self.alive(function_name)?;
        pop_self(&mut multivalue, function_name)?;

        let buffy = match multivalue.pop_front() {
            Some(LuaValue::Buffer(buffy)) => buffy,
            Some(other) => {
                return wrap_err!("{} expected target to be a buffer, got: {:?}", function_name, other);
            },
            None => {
                return wrap_err!("{} expected target buffer, but was incorrectly called with 0 arguments", function_name);
            }
        };

        let offset = match multivalue.pop_front() {
            Some(LuaValue::Integer(offset)) => int_to_usize(offset, function_name, "offset")?,
            Some(LuaValue::Number(f)) => float_to_usize(f, function_name, "offset")?,
            Some(LuaNil) | None => 0,
            Some(other) => {
                return wrap_err!("{} expected offset to be a number or nil, got: {:?}", function_name, other);
            }
        };

        let duration = match multivalue.pop_front() {
            Some(LuaValue::Number(f)) => Some(Self::check_valid_duration(f, function_name)?),
            Some(LuaValue::Integer(i)) => {
                let f = i as f64;
                Some(Self::check_valid_duration(f, function_name)?)
            },
            Some(LuaNil) | None => None,
            Some(other) => {
                return wrap_err!("{} expected duration to be a number or nil, got: {:?}", function_name, other);
            }
        };

        if offset >= buffy.len() {
            return wrap_err!("{}: offset {} >= buffer length {} (buffer would overflow); add an explicit offset < buffer_length check", function_name, offset, buffy.len());
        }

        self.sleep_if_needed(duration);

        let mut inner = self.inner.lock().unwrap();
        if inner.is_empty() {
            return Ok(LuaNil);
        }

        let last_index = {
            // we only want to drain as many bytes can fit into buffy since users can't specify how many bytes are expected up front
            // (use stream:readbytes_exact for that usecase instead)
            let space_left = buffy.len().saturating_sub(offset);
            // similarly, we don't want to try to read more bytes in inner than actually exist (causes a panic), so we must clamp to inner's length
            let max_index_in_inner = inner.len().saturating_sub(1);
            std::cmp::min(max_index_in_inner, space_left.saturating_sub(1))
        };

        let bytes_read : Vec<u8> = inner.drain(..=last_index).collect();
        if bytes_read.is_empty() {
            Ok(LuaNil)
        } else if offset + bytes_read.len() <= buffy.len() { // should've already been checked by precondition above but why not check again in case smth changed
            buffy.write_bytes(offset, &bytes_read);
            let byte_count: i64 = match bytes_read.len().try_into() {
                Ok(i) => i,
                Err(_) => {
                    return wrap_err!("{}: cannot convert the number of bytes read (usize) into i64");
                }
            };
            Ok(LuaValue::Integer(byte_count))
        } else {
            unreachable!(
                "{}: logic bug. drained more bytes than buffer space allowed (offset {} + drained {}) into buffer of size {}",
                function_name, offset, bytes_read.len(), buffy.len()
            )
        }
    }

    /// stream:readbytes_exact(count: number, target: buffer, offset: number?) -> (boolean, number?)
    /// 
    /// reads exactly `count` bytes into `target` at `offset` (0 if unspecified),
    /// - returns `(true, nil)` if `count` bytes are successfully read,
    /// - returns `(false, nil)` if the stream is empty (0 bytes read)
    /// - returns `(false, number)` if the stream isn't empty but we don't have enough exactly `count` bytes to read yet;
    ///   if this happens, 0 bytes are consumed so users are free to call this function again later without losing any data
    pub fn readbytes_exact(&mut self, luau: &Lua, mut multivalue: LuaMultiValue) -> LuaMultiResult {
        let function_name = match self.stream_type {
            StreamType::Stdout => "ChildProcess.stdout:readbytes_exact(count: number, target: buffer, offset: number?, duration: number?)",
            StreamType::Stderr => "ChildProcess.stderr:readbytes_exact(count: number, target: buffer, offset: number?, duration: number?)",
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

        let offset = match multivalue.pop_front() {
            Some(LuaValue::Integer(offset)) => int_to_usize(offset, function_name, "offset")?,
            Some(LuaValue::Number(f)) => float_to_usize(f, function_name, "offset")?,
            Some(LuaNil) | None => 0,
            Some(other) => {
                return wrap_err!("{} expected offset to be a number or nil, got: {:?}", function_name, other);
            }
        };

        let duration = match multivalue.pop_front() {
            Some(LuaValue::Number(f)) => Some(Self::check_valid_duration(f, function_name)?),
            Some(LuaValue::Integer(i)) => {
                let f = i as f64;
                Some(Self::check_valid_duration(f, function_name)?)
            },
            Some(LuaNil) | None => None,
            Some(other) => {
                return wrap_err!("{} expected duration to be a number or nil, got: {:?}", function_name, other);
            }
        };

        self.sleep_if_needed(duration);

        let mut inner = self.inner.lock().unwrap();
        if inner.is_empty() {
            // return success: `false` instead of erroring when stream is empty, otherwise we greatly annoy looping workflows
            LuaValue::Boolean(false).into_lua_multi(luau)
        } else if inner.len() < count {
            // return success: false, remaining: number instead of erroring so the user knows there's something in the buffer but
            // we couldn't read exactly the number of bytes they wanted, and that nothing was consumed
            vec![LuaValue::Boolean(false), inner.len().into_lua(luau)?].into_lua_multi(luau)
        } else if offset + count <= buffy.len() {
            let bytes_read: Vec<u8> = inner.drain(..count).collect();
            buffy.write_bytes(offset, &bytes_read);
            vec![
                LuaValue::Boolean(true),
                if inner.is_empty() {
                    LuaNil
                } else {
                    inner.len().into_lua(luau)?
                }
            ].into_lua_multi(luau)
        } else {
            wrap_err!("{}: can't fit offset {} + count {} bytes into buffer of length {}", function_name, offset, count, buffy.len())
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

    pub fn size(&mut self) -> LuaResult<usize> {
        let function_name = match self.stream_type {
            StreamType::Stdout => "ChildProcess.stdout:size()",
            StreamType::Stderr => "ChildProcess.stderr:size()",
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
}
