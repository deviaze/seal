use std::path::PathBuf;
use std::{fs, time::Duration};
use std::sync::Mutex;
use std::thread;

use regex::Regex;
use crossbeam_channel::{bounded, Sender, Receiver};

use crate::{table_helpers::TableBuilder, LuaValueResult, colors, globals, std_json};
use mlua::prelude::*;

fn thread_sleep(_luau: &Lua, duration: LuaNumber) -> LuaValueResult {
    let dur = Duration::from_millis(duration as u64);
    thread::sleep(dur);
    Ok(LuaValue::Boolean(true)) // ensure while thread.sleep(n) do end works
}

fn thread_spawn(luau: &Lua, spawn_options: LuaValue) -> LuaValueResult {
    match spawn_options {
        LuaValue::Table(options) => {
            let mut thread_src_path = String::from("");
            let mut thread_called_from_path = String::from("");
            let spawn_src = {
                if let LuaValue::String(src) = options.get("src")? {
                    let src = src.to_str()?.to_string();
                    Ok(src)
                } else if let LuaValue::String(path) = options.get("path")? {
                    let path = path.to_string_lossy();
                    let path = path
                        .trim_start_matches("./")
                        .trim_start_matches(".\\");
                    let called_from_path = match globals::get_script_path(luau, LuaMultiValue::new())? {
                        LuaValue::String(p) => p.to_string_lossy(),
                        other => {
                            return wrap_err!("called from path not a string??: {:?}", other);
                        }
                    };
                    thread_called_from_path = called_from_path.clone();
                    let parent_path = match globals::get_script_parent(luau, LuaMultiValue::new())? {
                        LuaValue::String(s) => s.to_string_lossy(),
                        other => {
                            return wrap_err!("thread parent path not string??: {:?}", other);
                        }
                    };
                    let mut thread_src_pathbuf = PathBuf::from(&parent_path);
                    thread_src_pathbuf.push(path);
                    thread_src_path = path.to_owned();
                    Ok(fs::read_to_string(thread_src_pathbuf).unwrap())
                } else {
                    wrap_err!("thread.spawn expected table with fields src or path, got neither")
                }
            }?;

            let spawn_data = {
                if let LuaValue::Table(data) = options.raw_get("data")? {
                    Some(std_json::json_encode(luau, LuaValue::Table(data).into_lua_multi(luau)?)?)
                } else {
                    None
                }
            };

            let spawn_data_buffer: Option<Vec<u8>> = {
                if let LuaValue::Buffer(data) = options.raw_get("data")? {
                    Some(data.to_vec())
                } else {
                    None
                }
            };

            let (parent_to_child_sender, parent_to_child_receiver): 
                (Sender<String>, Receiver<String>) = bounded(12);
            let parent_to_child_receiver_readawait_clone = parent_to_child_receiver.clone();

            let (parent_to_child_buffer_sender, parent_to_child_buffer_receiver): 
                (Sender<Vec<u8>>, Receiver<Vec<u8>>) = bounded(24);
            let parent_to_child_buffer_receiver_readawait_clone = parent_to_child_buffer_receiver.clone();

            let (child_to_parent_sender, child_to_parent_receiver): 
                (Sender<String>, Receiver<String>) = bounded(12);
            let child_to_parent_receiver_readawait_clone = child_to_parent_receiver.clone();

            let (child_to_parent_buffer_sender, child_to_parent_buffer_receiver): 
                (Sender<Vec<u8>>, Receiver<Vec<u8>>) = bounded(24);
            let child_to_parent_buffer_receiver_readawait_clone = child_to_parent_buffer_receiver.clone();

            let handle = thread::spawn(move || {
                let new_luau = mlua::Lua::new();

                globals::set_globals(&new_luau, String::default()).unwrap();

                new_luau.globals().raw_set("script",
                    TableBuilder::create(&new_luau).unwrap()
                        .with_value("current_path", thread_src_path).unwrap()
                        .with_value("thread_parent_path", thread_called_from_path).unwrap()
                        .with_value("src", spawn_src.to_owned()).unwrap()
                        .build().unwrap()
                ).unwrap();

                new_luau.globals().raw_set("channel",
                    TableBuilder::create(&new_luau).unwrap()
                        .with_function("read", move |new_luau: &Lua, _multivalue: LuaMultiValue| -> LuaValueResult {
                            match parent_to_child_receiver.try_recv() {
                                Ok(data) => {
                                    if data.starts_with("{") {
                                        // it's some json we have to decode and return
                                        let json_result = match std_json::json_decode(new_luau, data) {
                                            Ok(value) => value,
                                            Err(err) => {
                                                return wrap_err!("channel:read(): error decoding json: {}", err);
                                            }
                                        };
                                        Ok(json_result)
                                        // let json_result = std_json::json_decode(new_luau, data)?;
                                    } else {
                                        data.into_lua(new_luau)
                                    }
                                },
                                Err(_) => Ok(LuaNil)
                            }
                        }).unwrap()
                        .with_function("readbytes", move |luau: &Lua, _multivalue: LuaMultiValue| -> LuaValueResult {
                            match parent_to_child_buffer_receiver.try_recv() {
                                Ok(data) => {
                                    Ok(LuaValue::Buffer(
                                        luau.create_buffer(data).unwrap()
                                    ))
                                },
                                Err(_) => Ok(LuaNil)
                            }
                        }).unwrap()
                        .with_function("readbytes_await", move |luau: &Lua, _multivalue: LuaMultiValue| -> LuaValueResult {
                            match parent_to_child_buffer_receiver_readawait_clone.recv() {
                                Ok(data) => {
                                    Ok(LuaValue::Buffer(
                                        luau.create_buffer(data).unwrap()
                                    ))
                                },
                                Err(_) => Ok(LuaNil)
                            }
                        }).unwrap()
                        .with_function("read_await", move |luau: &Lua, _multivalue: LuaMultiValue| -> LuaValueResult {
                            match parent_to_child_receiver_readawait_clone.recv() {
                                Ok(data) => {
                                    if data.starts_with("{") {
                                        let json_result = match std_json::json_decode(luau, data) {
                                            Ok(value) => value,
                                            Err(err) => {
                                                return wrap_err!("channel:read_await(): error decoding json: {}", err);
                                            }
                                        };
                                        Ok(json_result)
                                    } else {
                                        data.into_lua(luau)
                                    }
                                },
                                Err(err) => {
                                    wrap_err!("channel:read_await: RecvError: {}", err)
                                }
                            }
                        }).unwrap()
                        .with_function("sendbytes", move |_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                            let _channel = match multivalue.pop_front() {
                                Some(LuaValue::Table(channel)) => channel,
                                Some(LuaValue::Buffer(_)) =>
                                    return wrap_err!("channel:sendbytes expected self to be a table, got buffer. Did you accidentally call this method with a . instead of : ?"),
                                Some(other) =>
                                    return wrap_err!("channel:sendbytes expected self, got: {:?}", other),
                                None => {
                                    return wrap_err!("channel:sendbytes(buffer: buffer) was called with nothing.");
                                }
                            };

                            let datavec = match multivalue.pop_back() {
                                Some(LuaValue::Buffer(buffy)) => buffy.to_vec(),
                                Some(other) => {
                                    return wrap_err!("channel:sendbytes expected a byte buffer, got: {:?}", other)
                                },
                                None => {
                                    return wrap_err!("channel:sendbytes(buffer: buffer) didn't get a buffer.")
                                }
                            };

                            match child_to_parent_buffer_sender.send(datavec) {
                                Ok(()) => Ok(LuaNil),
                                Err(err) => {
                                    wrap_err!("channel:sendbytes hit a SendError: {}", err)
                                }
                            }
                        }).unwrap()
                        .with_function("send", move |new_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                            let send_data = match multivalue.pop_back() {
                                Some(data) => {
                                    match data {
                                        LuaValue::Table(data) => {
                                            std_json::json_encode(new_luau, LuaValue::Table(data).into_lua_multi(new_luau)?)?
                                        },
                                        LuaValue::String(data) => {
                                            data.to_str()?.to_string()
                                        },
                                        other => {
                                            return wrap_err!("channel:send() (in thread) expected string or json-serializable data, got: {:?}", other);
                                        }
                                    }
                                },
                                None => {
                                    return wrap_err!("channel:send() (in thread) expected some json-serializable data, got nothing");
                                }
                            };
                            match child_to_parent_sender.send(send_data) {
                                Ok(()) => Ok(LuaNil),
                                Err(err) => {
                                    wrap_err!("channel:send() (in thread) Unable to send data: {}", err)
                                }
                            }
                        }).unwrap()
                        .with_value("data", {
                            if let Some(data) = spawn_data {
                                std_json::json_decode(&new_luau, data).unwrap()
                            } else if let Some(buffy_data) = spawn_data_buffer {
                                LuaValue::Buffer(new_luau.create_buffer(buffy_data).unwrap())
                            } else {
                                LuaNil
                            }
                        }).unwrap()
                        .build().unwrap()
                ).unwrap();

                match new_luau.load(spawn_src).exec() {
                    Ok(_) => {},
                    Err(err) => {
                        let replace_main_re = Regex::new(r#"\[string \"[^\"]+\"\]"#).unwrap();
                        let globals = new_luau.globals();
                        let script: LuaTable = globals.get("script").unwrap();
                        let current_path: String = script.get("current_path").unwrap();
                        let thread_parent_path: String = script.get("thread_parent_path").unwrap();
                        let err_context: Option<String> = script.get("context").unwrap();
                        let err_message = {
                            let err_message = replace_main_re
                                .replace_all(&err.to_string(), format!("[\"{}\"]", current_path))
                                .replace("_G.error", "error")
                                .to_string();
                            if let Some(context) = err_context {
                                let context = format!("{}[CONTEXT] {}{}{}\n", colors::BOLD_RED, context, colors::RESET, colors::RED);
                                context + &err_message + &format!("\n THREAD CALLED FROM: {}", thread_parent_path)
                            } else {
                                err_message + &format!("\n{}THREAD CALLED FROM:{} [\"{}\"]", colors::BOLD_RED, colors::RESET, thread_parent_path)
                            }
                        };
                        panic!("{}", err_message);
                    }
                }
            });
            // hmm wonder why this works, got it off copilot but yay fearful concurrency :p 
            // let arc_handle = Arc::new(Mutex::new(Some(handle)));
            let arc_handle = Mutex::new(Some(handle));

            Ok(LuaValue::Table(
                TableBuilder::create(luau)?
                    .with_function("send", move |luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                        let value = match multivalue.pop_back() {
                            Some(value) => value,
                            None => {
                                return wrap_err!("thread.send expected value, got nothing");
                            }
                        };
                        let send_data: String = {
                            match value {
                                LuaValue::String(s) => {
                                    s.to_str()?.to_string()
                                },
                                LuaValue::Table(data) => {
                                    match std_json::json_encode(luau, LuaValue::Table(data).into_lua_multi(luau)?) {
                                        Ok(json) => json,
                                        Err(err) => {
                                            return wrap_err!("thread.send unable to encode data to json: {}", err)
                                        }
                                    }
                                },
                                other => {
                                    return wrap_err!("thread.send expected string or table (to stringify as json), got {:?}", other);
                                }
                            }
                        };
                        match parent_to_child_sender.send(send_data) {
                            Ok(()) => {},
                            Err(err) => {
                                return wrap_err!("Some SendError occured: {}", err);
                            }
                        }
                        Ok(LuaNil)
                    })?
                    .with_function("sendbytes", move |_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                        let _channel = match multivalue.pop_front() {
                            Some(LuaValue::Table(channel)) => channel,
                            Some(LuaValue::Buffer(_)) =>
                                return wrap_err!("channel:sendbytes expected self to be a table, got buffer. Did you accidentally call this method with a . instead of : ?"),
                            Some(other) =>
                                return wrap_err!("channel:sendbytes expected self, got: {:?}", other),
                            None => {
                                return wrap_err!("channel:sendbytes(buffer: buffer) was called with nothing.");
                            }
                        };

                        let datavec = match multivalue.pop_back() {
                            Some(LuaValue::Buffer(buffy)) => buffy.to_vec(),
                            Some(other) => {
                                return wrap_err!("channel:sendbytes expected a byte buffer, got: {:?}", other)
                            },
                            None => {
                                return wrap_err!("channel:sendbytes(buffer: buffer) didn't get a buffer.")
                            }
                        };

                        match parent_to_child_buffer_sender.send(datavec) {
                            Ok(()) => Ok(LuaNil),
                            Err(err) => {
                                wrap_err!("channel:sendbytes hit a SendError: {}", err)
                            }
                        }
                    })?
                    .with_function("read", move |luau: &Lua, _multivalue: LuaMultiValue| -> LuaValueResult {
                        match child_to_parent_receiver.try_recv() {
                            Ok(data) => {
                                if data.starts_with("{") {
                                    // it's some json we have to decode and return
                                    let json_result = match std_json::json_decode(luau, data) {
                                        Ok(value) => value,
                                        Err(err) => {
                                            return wrap_err!("channel:read(): error decoding json: {}", err);
                                        }
                                    };
                                    Ok(json_result)
                                } else {
                                    data.into_lua(luau)
                                }
                            },
                            Err(_) => Ok(LuaNil)
                        }
                    })?
                    .with_function("read_await", move |luau: &Lua, _multivalue: LuaMultiValue| -> LuaValueResult {
                        match child_to_parent_receiver_readawait_clone.recv() {
                            Ok(data) => {
                                if data.starts_with("{") {
                                    let json_result = match std_json::json_decode(luau, data) {
                                        Ok(value) => value,
                                        Err(err) => {
                                            return wrap_err!("channel:read_await(): error decoding json: {}", err);
                                        }
                                    };
                                    Ok(json_result)
                                } else {
                                    data.into_lua(luau)
                                }
                            },
                            Err(err) => {
                                wrap_err!("channel:read_await: RecvError: {}", err)
                            }
                        }
                    })?
                    .with_function("readbytes", move |luau: &Lua, _multivalue: LuaMultiValue| -> LuaValueResult {
                        match child_to_parent_buffer_receiver.try_recv() {
                            Ok(data) => {
                                Ok(LuaValue::Buffer(
                                    luau.create_buffer(data)?
                                ))
                            },
                            Err(_) => Ok(LuaNil)
                        }
                    })?
                    .with_function("readbytes_await", move |luau: &Lua, _multivalue: LuaMultiValue| -> LuaValueResult {
                        match child_to_parent_buffer_receiver_readawait_clone.recv() {
                            Ok(data) => {
                                Ok(LuaValue::Buffer(
                                    luau.create_buffer(data)?
                                ))
                            },
                            Err(_) => Ok(LuaNil)
                        }
                    })?
                    .with_function("join", move |_luau: &Lua, _value: LuaValue| -> LuaValueResult {
                        let mut handle = arc_handle.lock().unwrap();
                        if let Some(handle) = handle.take() {
                            match handle.join() {
                                Ok(_) => {
                                    return Ok(LuaNil);
                                },
                                Err(_) => {
                                    return wrap_err!("error in called thread.spawn");
                                }
                            }
                        }
                        Ok(LuaNil)
                    })?
                    .build_readonly()?
            ))
        },
        other => {
            wrap_err!("thread.spawn: expected ThreadSpawnOptions table, got: {:?}", other)
        }
    }
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("spawn", thread_spawn)?
        .with_function("sleep", thread_sleep)?
        .build_readonly()
}