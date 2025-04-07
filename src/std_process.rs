use std::io::{BufRead, BufReader, Read, Write};
use std::process::{self, Command, Stdio};
// use std::thread;
use std::sync::{Arc, Mutex};

use mlua::prelude::*;
use crate::require::ok_table;
use crate::{std_env, colors, table_helpers::TableBuilder, wrap_err, LuaValueResult};

struct RunOptions {
    program: String,
    args: Vec<String>,
    shell: Option<String>,
}

impl RunOptions {
    #[allow(dead_code)]
    fn new(program: String, args: Vec<String>, shell: Option<String>) -> Self {
        RunOptions {
            program,
            args,
            shell
        }
    }

    fn from_table(luau: &Lua, run_options: LuaTable) -> LuaResult<Self> {
        let program = match run_options.raw_get("program").unwrap() {
            LuaValue::String(program) => {
                program.to_string_lossy()
            },
            LuaValue::Nil => {
                return wrap_err!("RunOptions missing field `program`; expected string, got nil");
            },
            other => {
                return wrap_err!("RunOptions.program expected to be a string, got: {:#?}", other);
            }
        };

        let args = match run_options.raw_get("args").unwrap() {
            LuaValue::Table(args) => {
                let mut rust_vec: Vec<String> = Vec::from_lua(LuaValue::Table(args), luau)?;
                // let's trim the whitespace just to make sure we pass valid args (untrimmed args might explode)
                for s in rust_vec.iter_mut() {
                    *s = s.trim().to_string();
                };
                rust_vec
            },
            LuaValue::Nil => {
                Vec::new()
            },
            other => {
                return wrap_err!("RunOptions.args expected to be {{string}} or nil, got: {:#?}", other);
            }
        };

        let shell = match run_options.raw_get("shell").unwrap() {
            LuaValue::String(shell) => {
                Some(shell.to_string_lossy())
            },
            LuaValue::Nil => {
                None
            },
            other => {
                return wrap_err!("RunOptions.shell expected to be a string or nil, got: {:#?}", other);
            }
        };

        Ok(RunOptions {
            program,
            args,
            shell
        })
        
    }
}

fn process_run(luau: &Lua, run_options: LuaValue) -> LuaValueResult {
    let options = match run_options {
        LuaValue::Table(run_options) => {
            RunOptions::from_table(luau, run_options)?
        },
        LuaValue::Nil => {
            return wrap_err!("process.run expected RunOptions table of type {{ program: string, args: {{string}}?, shell: string? }}, got nil.");
        },
        other => {
            return wrap_err!("process.run expected RunOptions table of type {{ program: string, args: {{string}}?, shell: string? }}, got: {:#?}", other);
        }
    };

    let output = {
        match if let Some(shell) = options.shell {
            Command::new(shell.clone())
                .arg(
                    if shell.as_str() == "pwsh" || shell.as_str() == "powershell" {
                        "-Command"
                    } else {
                        "-c"
                    }
                )
                .arg(options.program)
                .arg(options.args.join(" "))
                .output()
        } else {
            Command::new(options.program)
                .args(options.args)
                .output()
        } {
            Ok(output) => output,
            Err(err) => {
                // it breaks here on windows
                let err_message = format!("process.run failed to execute process: {}", err);
                let failed_to_exec_table = TableBuilder::create(luau)?
                    .with_value("ok", false)?
                    .with_value("out", "")?
                    .with_value("err", err_message)?
                    .with_value("stdout", "")?
                    .with_value("stderr", "")?
                    .with_function("unwrap",
                        | _luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                            let _failure_table = multivalue.pop_front();
                            let default_arg = match multivalue.pop_front() {
                                Some(value) => value,
                                None => {
                                    return wrap_err!("Attempt to ProcessRunResult:unwrap() an erred process.run without a default value!")
                                }
                            };
                            Ok(default_arg)
                        }
                    )?
                    .build_readonly();
                return ok_table(failed_to_exec_table)
            }
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        let success_table = TableBuilder::create(luau)?
            .with_value("ok", true)?
            .with_value("out", stdout.clone())?
            .with_value("stdout", stdout.clone())?
            .with_value("stderr", stderr.clone())?
            .with_function("unwrap", {
                let stdout = stdout.clone().to_string();
                move | luau: &Lua, _multivalue: LuaMultiValue | -> LuaValueResult {
                    Ok(LuaValue::String(luau.create_string(stdout.clone())?))
                }
            })?
            .build_readonly()?;
        Ok(LuaValue::Table(success_table))
    } else {
        let failure_table = TableBuilder::create(luau)?
            .with_value("ok", false)?
            .with_value("err", stderr.clone())?
            .with_value("stdout", stdout.clone())?
            .with_value("stderr", stderr.clone())?
            .with_function("unwrap",
                | _luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                    let _failure_table = multivalue.pop_front();
                    let default_arg = match multivalue.pop_front() {
                        Some(value) => value,
                        None => {
                            return wrap_err!("Attempt to ProcessRunResult:unwrap() an erred process.run without a default value!")
                        }
                    };
                    Ok(default_arg)
                }
            )?
            .build_readonly()?;
        Ok(LuaValue::Table(failure_table))
    }
}

fn process_spawn(luau: &Lua, spawn_options: LuaValue) -> LuaValueResult {
    let options = match spawn_options {
        LuaValue::Table(run_options) => {
            RunOptions::from_table(luau, run_options)?
        },
        LuaValue::Nil => {
            return wrap_err!("process.spawn expected RunOptions table of type {{ program: string, args: {{string}}?, shell: string? }}, got nil.");
        },
        other => {
            return wrap_err!("process.spawn expected RunOptions table of type {{ program: string, args: {{string}}?, shell: string? }}, got: {:#?}", other);
        }
    };

    let mut child = {
        match if let Some(shell) = options.shell {
            Command::new(shell.clone())
                .arg(
                    if shell.as_str() == "pwsh" || shell.as_str() == "powershell" {
                        "-Command"
                    } else {
                        "-c"
                    }
                )
                .arg(options.program)
                .arg(options.args.join(" "))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        } else {
            Command::new(options.program)
                .args(options.args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        } {
            Ok(child) => child,
            Err(err) => {
                return wrap_err!("process.spawn failed to execute process: {}", err);
            }
        }
    };

    let child_id = child.id();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let stdin = child.stdin.take().unwrap();

    let arc_child = Arc::new(Mutex::new(child));
    let arc_stdout = Arc::new(Mutex::new(stdout));
    let arc_stderr = Arc::new(Mutex::new(stderr));
    let arc_stdin = Arc::new(Mutex::new(stdin));

    let stdout_handle = TableBuilder::create(luau)?
        .with_function("read", {
            let stdout = Arc::clone(&arc_stdout);
            move | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                let buffer_size = match multivalue.pop_back() {
                    Some(LuaValue::Integer(i)) => i as usize,
                    Some(LuaValue::Number(n)) => {
                        if n.trunc() == n {
                            n as usize
                        } else {
                            return wrap_err!("ChildProcess.stdout:read(buffer_size) expected buffer_size to be an integer, got a float: {}", n);
                        }
                    },
                    _ => 32,
                };
                let mut stdout = stdout.lock().unwrap();
                let mut buffy = vec![0; buffer_size];
                match stdout.read_exact(&mut buffy) {
                    Ok(_) => {
                        let result_string = luau.create_string(buffy)?;
                        Ok(LuaValue::String(result_string))
                    },
                    Err(_err) => Ok(LuaValue::Nil)
                }
            }
        })?
        .with_function("lines", {
            let stdout = Arc::clone(&arc_stdout);
            move | luau: &Lua, _multivalue: LuaMultiValue | -> LuaValueResult {
                Ok(LuaValue::Function(luau.create_function({
                    let stdout = Arc::clone(&stdout);
                    move | luau: &Lua, _value: LuaValue | -> LuaValueResult {
                        let mut stdout = stdout.lock().unwrap();
                        let mut reader = BufReader::new(stdout.by_ref());
                        let mut new_line = String::from("");
                        match reader.read_line(&mut new_line) {
                            Ok(0) => {
                                Ok(LuaNil)
                            },
                            Ok(_other) => {
                                Ok(LuaValue::String(luau.create_string(new_line.trim_end())?))
                            },
                            Err(err) => {
                                wrap_err!("unable to read line: {:#?}", err)
                            }
                        }
                    }
                })?))
                // let line = reader.read_line(buf)
            }
        })?
        .build_readonly()?;

    let stderr_handle = TableBuilder::create(luau)?
        .with_function("read", {
            let stderr = Arc::clone(&arc_stderr);
            move | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                let buffer_size = match multivalue.pop_back() {
                    Some(LuaValue::Integer(i)) => i as usize,
                    Some(LuaValue::Number(n)) => {
                        if n.trunc() == n {
                            n as usize
                        } else {
                            return wrap_err!("ChildProcess.stderr:read(buffer_size) expected buffer_size to be an integer, got a float: {}", n);
                        }
                    },
                    _ => 32,
                };
                let mut stderr = stderr.lock().unwrap();
                let mut buffy = vec![0; buffer_size];
                match stderr.read_exact(&mut buffy) {
                    Ok(_) => {
                        let result_string = luau.create_string(buffy)?;
                        Ok(LuaValue::String(result_string))
                    },
                    Err(_err) => Ok(LuaValue::Nil)
                }
            }
        })?
        .with_function("lines", {
            let stderr = Arc::clone(&arc_stderr);
            move | luau: &Lua, _multivalue: LuaMultiValue | -> LuaValueResult {
                Ok(LuaValue::Function(luau.create_function({
                    let stderr = Arc::clone(&stderr);
                    move | luau: &Lua, _value: LuaValue | -> LuaValueResult {
                        let mut stderr = stderr.lock().unwrap();
                        let mut reader = BufReader::new(stderr.by_ref());
                        let mut new_line = String::from("");
                        match reader.read_line(&mut new_line) {
                            Ok(0) => {
                                Ok(LuaNil)
                            },
                            Ok(_other) => {
                                Ok(LuaValue::String(luau.create_string(new_line.trim_end())?))
                            },
                            Err(err) => {
                                wrap_err!("unable to read line: {:#?}", err)
                            }
                        }
                    }
                })?))
                // let line = reader.read_line(buf)
            }
        })?
        .build_readonly()?;

    let stdin_handle = TableBuilder::create(luau)?
        .with_function("write", {
            let stdin = Arc::clone(&arc_stdin);
            move | _luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                let _handle = multivalue.pop_front();
                let stuff_to_write = match multivalue.pop_back() {
                    Some(LuaValue::String(stuff)) => stuff.to_string_lossy(),
                    Some(other) => {
                        return wrap_err!("ChildProcess.stdin:write(data) expected data to be a string, got: {:?}", other);
                    },
                    None => {
                        return wrap_err!("ChildProcess.stdin:write(data) was called without argument data");
                    }
                };
                let mut stdin = stdin.lock().unwrap();
                match stdin.write_all(stuff_to_write.as_bytes()) {
                    Ok(_) => Ok(LuaNil),
                    Err(err) => wrap_err!("ChildProcess.stdin:write: error writing to stdin: {:?}", err)
                }
            }
        })?
        .build_readonly()?;
    
    let child_handle = TableBuilder::create(luau)?
        .with_value("id", child_id)?
        .with_function("alive",{
            let child = Arc::clone(&arc_child);
            move | _luau: &Lua, _multivalue: LuaMultiValue | -> LuaValueResult {
                let mut child = child.lock().unwrap();
                match child.try_wait().unwrap() {
                    Some(_status_code) => Ok(LuaValue::Boolean(false)),
                    None => Ok(LuaValue::Boolean(true)),
                }
            }
        })?
        .with_function("kill",{
            let child = Arc::clone(&arc_child);
            move | _luau: &Lua, _multivalue: LuaMultiValue | -> LuaValueResult {
                let mut child = child.lock().unwrap();
                match child.kill() {
                    Ok(_) => Ok(LuaValue::Nil),
                    Err(err) => {
                        wrap_err!("ChildProcess could not be killed: {:?}", err)
                    }
                }
            }
        })?
        .with_value("stdout", LuaValue::Table(stdout_handle))?
        .with_value("stderr", stderr_handle)?
        .with_value("stdin", stdin_handle)?
        .build_readonly()?;

    Ok(LuaValue::Table(child_handle))
}

fn process_shell(luau: &Lua, shell_command: LuaValue) -> LuaValueResult {
    let shell_path = std_env::get_current_shell();
    match shell_command {
        LuaValue::String(command) => {
            process_run(luau, LuaValue::Table(
                TableBuilder::create(luau)?
                    .with_value("program", command)?
                    .with_value("shell", shell_path)?
                    .build_readonly()?
            ))
        },
        other => {
            wrap_err!("process.shell(command) expected command to be a string, got: {:#?}", other)
        }
    }
}

fn set_exit_callback(luau: &Lua, f: Option<LuaValue>) -> LuaValueResult {
    if let Some(f) = f {
        match f {
            LuaValue::Function(f) => {
                let globals = luau.globals();
                globals.set("_process_exit_callback_function", f)?;
                Ok(LuaNil)
            }, 
            _ => {
                let err_message = format!("process.setexitcallback expected to be called with a function, got {:?}", f);
                Err(LuaError::external(err_message))
            }
        }
    } else {
        let err_message = format!("process.setexitcallback expected to be called with a function, got {:?}", f);
        Err(LuaError::external(err_message))
    }
}

pub fn handle_exit_callback(luau: &Lua, exit_code: i32) -> LuaResult<()> {
    match luau.globals().get("_process_exit_callback_function")? {
        LuaValue::Function(f) => {
            let _ = f.call::<i32>(exit_code);
        },
        LuaValue::Nil => {},
        _ => {
            unreachable!("what did you put into _process_exit_callback_function???");
        }
    }
    Ok(())
}

fn exit(luau: &Lua, exit_code: Option<LuaValue>) -> LuaResult<()> {
    let exit_code = if let Some(exit_code) = exit_code {
        match exit_code {
            LuaValue::Integer(i) => i,
            _ => {
                return wrap_err!("process.exit expected exit_code to be a number (integer) or nil, got {:?}", exit_code);
            }
        }
    } else {
        0
    };
    // if we have custom callback function let's call it 
    let globals = luau.globals();
    match globals.get("_process_exit_callback_function")? {
        LuaValue::Function(f) => {
            f.call::<i32>(exit_code)?;
        },
        LuaValue::Nil => {},
        other => {
            unreachable!("wtf is in _process_exit_callback_function other than a function or nil?: {:?}", other)
        }
    }
    process::exit(exit_code);
}


pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("run", process_run)?
        .with_function("spawn", process_spawn)?
        .with_function("shell", process_shell)?
        .with_function("setexitcallback", set_exit_callback)?
        .with_function("exit", exit)?
        .build_readonly()
}
