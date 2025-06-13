use std::time::Duration;
use std::sync::Mutex;
use std::thread;

use crate::prelude::*;
use crate::{std_json, globals, err};
use crossbeam_channel::TrySendError;
use mlua::prelude::*;

mod channel;
mod thread_spawn_options;

use thread_spawn_options::ThreadSpawnOptions;
use channel::Channel;

fn thread_sleep(_luau: &Lua, duration: LuaNumber) -> LuaValueResult {
    let dur = Duration::from_millis(duration as u64);
    thread::sleep(dur);
    Ok(LuaValue::Boolean(true)) // ensure while thread.sleep(n) do end works
}

struct Channels {
    parent_to_child: Channel<String>,
    parent_to_child_bytes: Channel<Vec<u8>>,
    child_to_parent: Channel<String>,
    child_to_parent_bytes: Channel<Vec<u8>>,
}

fn thread_spawn(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "thread.spawn(options: ThreadSpawnOptions)";
    let options = match value {
        LuaValue::Table(t) => ThreadSpawnOptions::from_table(t, luau, function_name)?,
        other => {
            return wrap_err!("{} expected options to be a ThreadSpawnOptions table (with fields path or src and optionally data), got: {:?}", function_name, other);
        }
    };

    let thread_name = options.name.clone();
    let src = options.get_src(function_name)?;

    let channels = Channels {
        parent_to_child: Channel::new(options.capacity.regular),
        parent_to_child_bytes: Channel::new(options.capacity.bytes),
        child_to_parent: Channel::new(options.capacity.regular),
        child_to_parent_bytes: Channel::new(options.capacity.bytes),
    };

    let thread_builder = thread::Builder::new()
        .name(options.name.clone());

    let join_handle_result = thread_builder.spawn(move || -> LuaEmptyResult {
        let new_luau = Lua::default();
        let data = match options.data {
            Some(data) => deserialize_data_from_transit(&new_luau, data)?,
            None => LuaNil,
        };
        
        globals::set_globals(&new_luau, options.chunk_name.clone())?;
        // must use globals.set() due to safeenv
        new_luau.globals().set("channel", TableBuilder::create(&new_luau)?
            .with_function("read", {
                let receiver = channels.parent_to_child.receiver.clone();
                move | luau: &Lua, _value: LuaValue | -> LuaValueResult {
                    let function_name = "channel:read()";
                    match receiver.try_recv(function_name)? {
                        Some(data) => deserialize_data_from_transit(luau, data),
                        None => Ok(LuaNil),
                    }
                }
            })?
            .with_function("read_await", {
                let receiver = channels.parent_to_child.receiver;
                move | luau: &Lua, _value: LuaValue | -> LuaValueResult {
                    let function_name = "channel:read_await()";
                    match receiver.recv_await(function_name) {
                        Ok(data) => deserialize_data_from_transit(luau, data),
                        Err(err) => Err(err)
                    }
                }
            })?
            .with_function("readbytes", {
                let receiver = channels.parent_to_child_bytes.receiver.clone();
                move | luau: &Lua, _value: LuaValue | -> LuaValueResult {
                    let function_name = "channel:readbytes()";
                    match receiver.try_recv(function_name)? {
                        Some(data) => ok_buffy(data, luau),
                        None => Ok(LuaNil)
                    }
                }
            })?
            .with_function("readbytes_await", {
                let receiver = channels.parent_to_child_bytes.receiver;
                move | luau: &Lua, _value: LuaValue | -> LuaValueResult {
                    let function_name = "channel:readbytes_await()";
                    match receiver.recv_await(function_name) {
                        Ok(data) => ok_buffy(data, luau),
                        Err(err) => Err(err)
                    }
                }
            })?
            .with_function("send", {
                let sender = channels.child_to_parent.sender.clone();
                move | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaEmptyResult {
                    let function_name = "channel:send(data: string | JsonSerializableTable)";
                    let _s = pop_self(&mut multivalue, function_name)?;
                    let value = match multivalue.pop_front() {
                        Some(v) => v,
                        None => {
                            return wrap_err!("{} called without required argument 'data'", function_name);
                        }
                    };
                    let data = serialize_data_for_transit(luau, value, function_name)?;
                    sender.send(data, function_name)
                }
            })?
            .with_function("try_send", {
                let sender = channels.child_to_parent.sender.clone();
                move | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaMultiResult {
                    let function_name = "channel:try_send(data: string | JsonSerializableTable)";
                    let _s = pop_self(&mut multivalue, function_name)?;
                    let value = match multivalue.pop_front() {
                        Some(v) => v,
                        None => {
                            return wrap_err!("{} called without 'data' (expected string or JsonSerializableTable, got nothing)", function_name);
                        }
                    };
                    let data = serialize_data_for_transit(luau, value, function_name)?;
                    match sender.try_send(data) {
                        Ok(_) => {
                            let success = true;
                            let multi = LuaMultiValue::from_vec(vec![
                                LuaValue::Boolean(success),
                                LuaValue::String(luau.create_string("Sent")?),
                            ]);
                            Ok(multi)
                        },
                        Err(err) => {
                            let success = false;
                            let result = match err {
                                TrySendError::Disconnected(_) => "Disconnected",
                                TrySendError::Full(_) => "Full",
                            };
                            let multi = LuaMultiValue::from_vec(vec![
                                LuaValue::Boolean(success),
                                LuaValue::String(luau.create_string(result)?),
                            ]);
                            Ok(multi)
                        }
                    }
                }
            })?
            .with_function("sendbytes", {
                let sender = channels.child_to_parent_bytes.sender.clone();
                move | _luau: &Lua, mut multivalue: LuaMultiValue | -> LuaEmptyResult {
                    let function_name = "channel:sendbytes(data: buffer)";
                    let _s = pop_self(&mut multivalue, function_name)?;
                    let data = match multivalue.pop_front() {
                        Some(LuaValue::Buffer(buffy)) => buffy.to_vec(),
                        Some(other) => {
                            return wrap_err!("{} expected data to be a buffer, got: {:?}", function_name, other);
                        },
                        None => {
                            return wrap_err!("{} called without required argument 'data'", function_name);
                        }
                    };
                    sender.send(data, function_name) 
                }
            })?
            .with_function("try_sendbytes", {
                let sender = channels.child_to_parent_bytes.sender;
                move | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaMultiResult {
                    let function_name = "channel:try_sendbytes(data: buffer)";
                    let _s = pop_self(&mut multivalue, function_name)?;
                    let data = match multivalue.pop_front() {
                        Some(LuaValue::Buffer(buffy)) => buffy.to_vec(),
                        Some(other) => {
                            return wrap_err!("{} expected data to be a buffer, got: {:?}", function_name, other);
                        }
                        None => {
                            return wrap_err!("{} called without 'data' (expected buffer, got nothing)", function_name);
                        }
                    };
                    match sender.try_send(data) {
                        Ok(_) => {
                            let success = true;
                            let multi = LuaMultiValue::from_vec(vec![
                                LuaValue::Boolean(success),
                                LuaValue::String(luau.create_string("Sent")?),
                            ]);
                            Ok(multi)
                        },
                        Err(err) => {
                            let success = false;
                            let result = match err {
                                TrySendError::Disconnected(_) => "Disconnected",
                                TrySendError::Full(_) => "Full",
                            };
                            let multi = LuaMultiValue::from_vec(vec![
                                LuaValue::Boolean(success),
                                LuaValue::String(luau.create_string(result)?),
                            ]);
                            Ok(multi)
                        }
                    }
                }
            })?
            .with_value("data", data)?
            .build_readonly()?
        )?;

        match new_luau.load(src).set_name(options.chunk_name).exec() {
            Ok(_) => Ok(()),
            Err(err) => {
                let formatted_err = LuaError::external(format!("{}{}{}\n Error occurred in thread '{}', which was spawned at {}", colors::RED, err, colors::RESET, thread_name, options.spawned_at));
                // if we dont exit the main program when the child thread errors then the child thread exits and
                // we get RecvError (receiving on empty + disconnected thread) on the main thread
                err::display_error_and_exit(formatted_err);
            },
        }
    });
    let handle = match join_handle_result {
        Ok(handle) => Mutex::new(Some(handle)),
        Err(err) => {
            return wrap_err!("{}: can't spawn thread due to io error: {}", function_name, err);
        }
    };

    let thread_handle = TableBuilder::create(luau)?
        .with_value("name", luau.create_string(options.name.clone())?)?
        .with_function("join", {
            let thread_name = options.name.clone();
            move | _luau: &Lua, _value: LuaValue | -> LuaEmptyResult {
                let function_name = "ThreadHandle:join()";

                let handle = match handle.try_lock() {
                    Ok(mut handle) => match handle.take() {
                        Some(handle) => handle,
                        None => {
                            return wrap_err!("{}: unable to join handle; thread '{}' already joined or missing", function_name, thread_name);
                        }
                    },
                    Err(err) => {
                        return wrap_err!("{}: unable to lock thread handle for joining: {}", function_name, err);
                    }
                };

                match handle.join() {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        wrap_err!("{}: unable to join Rust Thread '{}' due to err: {:?}", function_name, thread_name, err)
                    }
                }

            }
        })?
        .with_function("read", {
            let receiver = channels.child_to_parent.receiver.clone();
            move | luau: &Lua, _value: LuaValue | -> LuaValueResult {
                let function_name = "ThreadHandle:read()";
                match receiver.try_recv(function_name)? {
                    Some(data) => deserialize_data_from_transit(luau, data),
                    None => Ok(LuaNil),
                }
            }
        })?
        .with_function("read_await", {
            let receiver = channels.child_to_parent.receiver;
            move | luau: &Lua, _value: LuaValue | -> LuaValueResult {
                let function_name = "ThreadHandle:read_await()";
                match receiver.recv_await(function_name) {
                    Ok(data) => deserialize_data_from_transit(luau, data),
                    Err(err) => Err(err)
                }
            }
        })?
        .with_function("readbytes", {
            let receiver = channels.child_to_parent_bytes.receiver.clone();
            move | luau: &Lua, _value: LuaValue | -> LuaValueResult {
                let function_name = "ThreadHandle:readbytes()";
                match receiver.try_recv(function_name)? {
                    Some(data) => ok_buffy(data, luau),
                    None => Ok(LuaNil)
                }
            }
        })?
        .with_function("readbytes_await", {
            let receiver = channels.child_to_parent_bytes.receiver;
            move | luau: &Lua, _value: LuaValue | -> LuaValueResult {
                let function_name = "ThreadHandle:readbytes_await()";
                match receiver.recv_await(function_name) {
                    Ok(data) => ok_buffy(data, luau),
                    Err(err) => Err(err)
                }
            }
        })?
        .with_function("send", {
            let sender = channels.parent_to_child.sender.clone();
            move | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaEmptyResult {
                let function_name = "ThreadHandle:send(data: string | JsonSerializableTable)";
                let _s = pop_self(&mut multivalue, function_name)?;
                let value = match multivalue.pop_front() {
                    Some(v) => v,
                    None => {
                        return wrap_err!("{} called without 'data' (expected string or JsonSerializableTable, got nothing)", function_name);
                    }
                };
                let data = serialize_data_for_transit(luau, value, function_name)?;
                sender.send(data, function_name)
            }
        })?
        .with_function("try_send", {
            let sender = channels.parent_to_child.sender;
            move | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaMultiResult {
                let function_name = "ThreadHandle:try_send(data: string | JsonSerializableTable)";
                let _s = pop_self(&mut multivalue, function_name)?;
                let value = match multivalue.pop_front() {
                    Some(v) => v,
                    None => {
                        return wrap_err!("{} called without 'data' (expected string or JsonSerializableTable, got nothing)", function_name);
                    }
                };
                let data = serialize_data_for_transit(luau, value, function_name)?;
                match sender.try_send(data) {
                    Ok(_) => {
                        let success = true;
                        let multi = LuaMultiValue::from_vec(vec![
                            LuaValue::Boolean(success), 
                            LuaValue::String(luau.create_string("Sent")?),
                        ]);
                        Ok(multi)
                    },
                    Err(err) => {
                        let success = false;
                        let result = match err {
                            TrySendError::Disconnected(_) => "Disconnected",
                            TrySendError::Full(_) => "Full",
                        };
                        let multi = LuaMultiValue::from_vec(vec![
                            LuaValue::Boolean(success),
                            LuaValue::String(luau.create_string(result)?),
                        ]);
                        Ok(multi)
                    }
                }
            }
        })?
        .with_function("sendbytes", {
            let sender = channels.parent_to_child_bytes.sender.clone();
            move | _luau: &Lua, mut multivalue: LuaMultiValue | -> LuaEmptyResult {
                let function_name = "ThreadHandle:sendbytes(data: buffer)";
                let _s = pop_self(&mut multivalue, function_name)?;
                let bytes = match multivalue.pop_front() {
                    Some(LuaValue::Buffer(buffy)) => buffy.to_vec(),
                    Some(other) => {
                        return wrap_err!("{} expected data to be a buffer, got: {:?}", function_name, other);
                    },
                    None => {
                        return wrap_err!("{} called without required argument 'data'", function_name);
                    }
                };
                sender.send(bytes, function_name)
            }
        })?
        .with_function("try_sendbytes", {
            let sender = channels.parent_to_child_bytes.sender;
            move | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaMultiResult {
                let function_name = "ThreadHandle:try_sendbytes(data: buffer)";
                let _s = pop_self(&mut multivalue, function_name)?;
                let data = match multivalue.pop_front() {
                    Some(LuaValue::Buffer(buffy)) => buffy.to_vec(),
                    Some(other) => {
                        return wrap_err!("{} expected data to be a buffer, got: {:?}", function_name, other);
                    }
                    None => {
                        return wrap_err!("{} called without 'data' (expected buffer, got nothing)", function_name);
                    }
                };
                match sender.try_send(data) {
                    Ok(_) => {
                        let success = true;
                        let multi = LuaMultiValue::from_vec(vec![
                            LuaValue::Boolean(success),
                            LuaValue::String(luau.create_string("Sent")?),
                        ]);
                        Ok(multi)
                    },
                    Err(err) => {
                        let success = false;
                        let result = match err {
                            TrySendError::Disconnected(_) => "Disconnected",
                            TrySendError::Full(_) => "Full",
                        };
                        let multi = LuaMultiValue::from_vec(vec![
                            LuaValue::Boolean(success),
                            LuaValue::String(luau.create_string(result)?),
                        ]);
                        Ok(multi)
                    }
                }
            }
        })?
        .build_readonly();

    ok_table(thread_handle)
}

fn serialize_data_for_transit(luau: &Lua, value: LuaValue, function_name: &'static str) -> LuaResult<String> {
    let data = match value {
        LuaValue::Table(data) => {
            match std_json::encode(luau, data, std_json::EncodeOptions::default()) {
                Ok(data) => data,
                Err(err) => {
                    return wrap_err!("{}: unable to serialize table (to send across the wire) due to err: {}", function_name, err);
                }
            }
        },
        LuaValue::String(s) => s.to_str()?.to_string(),
        other => {
            return wrap_err!("{} expected data to be a string or json-serializable table, got: {:?}", function_name, other);
        }
    };
    Ok(data)
}

fn deserialize_data_from_transit(luau: &Lua, data: String) -> LuaValueResult {
    match std_json::json_decode(luau, data.clone()) {
        Ok(d) => Ok(d),
        Err(_) => ok_string(data, luau),
    }
}

fn pop_self(multivalue: &mut LuaMultiValue, function_name: &'static str) -> LuaResult<LuaTable> {
    match multivalue.pop_front() {
        Some(LuaValue::Table(t)) => Ok(t),
        Some(other) => {
            wrap_err!("{} expected to be called with self, got: {:?}", function_name, other)
        },
        None => {
            wrap_err!("{} expected to be called with self; did you forget to use methodcall (:) syntax?", function_name)
        }
    }
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("spawn", thread_spawn)?
        .with_function("sleep", thread_sleep)?
        .build_readonly()
}