use core::str;
use std::fmt::Debug;
use std::time::Instant;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output, Stdio};
// use std::thread;
use std::sync::{Arc, Mutex};

use mluau::prelude::*;
use crate::prelude::*;
use crate::std_env;

mod stream;

use stream::Stream;

#[derive(Debug)]
enum Shell {
    #[allow(clippy::enum_variant_names)]
    WindowsPowerShell,
    Pwsh,
    Bash,
    Sh,
    Zsh,
    Fish,
    CmdDotExe,
    Other(String),
}

impl From<String> for Shell {
    fn from(s: String) -> Self {
        let shell_name = Path::new(&s)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&s); // If file_name fails, fall back to the original

        match shell_name {
            "pwsh" => Shell::Pwsh,
            "powershell" => Shell::WindowsPowerShell,
            "bash" => Shell::Bash,
            "sh" => Shell::Sh,
            "zsh" => Shell::Zsh,
            "fish" => Shell::Fish,
            "cmd" | "cmd.exe" => Shell::CmdDotExe,
            other => Shell::Other(other.to_string()),
        }
    }
}

impl Shell {
    fn program_name(&self) -> &str {
        match self {
            Shell::Pwsh => "pwsh",
            Shell::WindowsPowerShell => "powershell",
            Shell::Bash => "bash",
            Shell::Sh => "sh",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::CmdDotExe => "cmd.exe",
            Shell::Other(name) => name.as_str(),
        }
    }
    fn get_switches(&self) -> Vec<&str> {
        match self {
            Shell::Pwsh | Shell::WindowsPowerShell => vec!["-Command", "-NonInteractive"],
            Shell::CmdDotExe => vec!["/C"],
            _other => vec!["-c"],
        }
    }
}

#[derive(Debug)]
struct RunOptions {
    program: String,
    args: Option<Vec<String>>,
    shell: Option<Shell>,
    cwd: Option<PathBuf>,
}

impl RunOptions {
    fn from_table(luau: &Lua, run_options: LuaTable) -> LuaResult<Self> {
        let program = match run_options.raw_get("program")? {
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

        let args = match run_options.raw_get("args")? {
            LuaValue::Table(args) => {
                let mut rust_vec: Vec<String> = Vec::from_lua(LuaValue::Table(args), luau)?;
                // let's trim the whitespace just to make sure we pass valid args (untrimmed args might explode)
                for s in rust_vec.iter_mut() {
                    *s = s.trim().to_string();
                };
                Some(rust_vec)
            },
            LuaValue::Nil => {
                None
            },
            other => {
                return wrap_err!("RunOptions.args expected to be {{string}} or nil, got: {:#?}", other);
            }
        };

        let shell = match run_options.raw_get("shell")? {
            LuaValue::String(shell) => {
                Some(Shell::from(shell.to_string_lossy()))
            },
            LuaValue::Nil => {
                None
            },
            other => {
                return wrap_err!("RunOptions.shell expected to be a string or nil, got: {:#?}", other);
            }
        };

        let cwd = match run_options.raw_get("cwd")? {
            LuaValue::String(cwd) => {
                let cwd = cwd.as_bytes();
                let cwd_str = str::from_utf8(&cwd)?;
                let cwd_pathpuf = PathBuf::from(cwd_str);
                let canonicalized_cwd = match cwd_pathpuf.canonicalize() {
                    Ok(pathbuf) => pathbuf,
                    Err(err) => {
                        return wrap_err!("RunOptions.cwd must be able to be canonicalized as an absolute path that currently exists on the filesystem; \
                        canonicalization failed with err: {}", err);
                    }
                };
                Some(canonicalized_cwd)
            },
            LuaNil => None,
            other => {
                return wrap_err!("RunOpitons.cwd to be a string or nil, got: {:?}", other);
            }
        };

        Ok(RunOptions {
            program,
            args,
            shell,
            cwd,
        })
        
    }
}

fn run_result_unwrap_or(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "RunResult:unwrap_or(default: string | (result: RunResult) -> string)";
    let run_result = match multivalue.pop_front() {
        Some(LuaValue::Table(run_result)) => run_result,
        Some(other) => {
            return wrap_err!("{} expected self to be a RunResult table, got: {:?}", function_name, other);
        }
        None => {
            return wrap_err!("{} expected to be called with self; did you forget methodcall syntax (:)?", function_name);
        },
    };

    let default_value: Option<LuaValue> = match multivalue.pop_front() {
        Some(LuaValue::String(default)) => Some(LuaValue::String(default)),
        Some(LuaValue::Function(f)) => Some(LuaValue::Function(f)),
        Some(LuaNil) => Some(LuaNil),
        Some(other) => {
            return wrap_err!("{}: expected default value to be a string (or a function that returns one), got: {:?}", function_name, other);
        },
        None => None,
    };

    let is_ok = match run_result.raw_get("ok")? {
        LuaValue::Boolean(b) => b,
        other => {
            return wrap_err!("{}: expected RunResult.ok to be a boolean, got: {:?}", function_name, other);
        },
    };

    let stdout = if is_ok {
        match run_result.raw_get("stdout")? {
            LuaValue::String(s) => {
                let Ok(s) = s.to_str() else {
                    return wrap_err!("{}: stdout is not a valid utf-8 encoded string, use RunResult.stdout to get the raw stdout without attempting to trim/clean it", function_name);
                };
                let s = s.trim_end();
                LuaValue::String(luau.create_string(s)?)
            },
            other => {
                return wrap_err!("{} RunResult.stdout is not a string??: {:?}", function_name, other);
            }
        }
    } else if let Some(default_value) = default_value {
        match default_value {
            LuaValue::String(d) => LuaValue::String(d),
            LuaValue::Function(f) => {
                match f.call::<LuaValue>(run_result) {
                    Ok(LuaValue::String(default)) => {
                        LuaValue::String(default)
                    },
                    Ok(other) => {
                        return wrap_err!("{}: expected default value function to return string, got: {:?}", function_name, other);
                    },
                    Err(err) => {
                        return wrap_err!("{}: default value function unexpectedly errored: {}", function_name, err);
                    },
                }
            },
            other => {
                return wrap_err!("{}: default value expected to be a string (or a function that returns one), got: {:?}", function_name, other);
            }
        }
    } else {
        return wrap_err!("Attempt to {} an unsuccessful RunResult without a default value!", function_name);
    };
    Ok(stdout)
}

fn trim_end_or_return(vec: &[u8]) -> &[u8] {
    match str::from_utf8(vec) {
        Ok(s) => s.trim_end().as_bytes(),
        Err(_) => vec
    }
}

fn create_run_result_table(luau: &Lua, output: Output) -> LuaValueResult {
    let ok = output.status.success();
    let stdout = output.stdout.clone();
    let stderr = output.stderr.clone();

    let run_result = TableBuilder::create(luau)?
        .with_value("ok", ok)?
        .with_value("out", {
            if ok {
                let s = trim_end_or_return(&stdout);
                LuaValue::String(luau.create_string(s)?)
            } else {
                LuaNil
            }
        })?
        .with_value("err", {
            if !ok {
                let s = trim_end_or_return(&stderr);
                LuaValue::String(luau.create_string(s)?)
            } else {
                LuaNil
            }
        })?
        .with_value("stdout", luau.create_string(&stdout)?)?
        .with_value("stderr", luau.create_string(&stderr)?)?
        .with_function("unwrap", {
            move | luau: &Lua, _value: LuaMultiValue | -> LuaValueResult {
                if ok {
                    let s = trim_end_or_return(&stdout);
                    Ok(LuaValue::String(luau.create_string(s)?))
                } else {
                    wrap_err!("Attempt to :unwrap() a failed RunResult! Use :unwrap_or to specify a default value")
                }
            }
        })?
        .with_function("unwrap_or", run_result_unwrap_or)?
        .build_readonly();

    ok_table(run_result)
}

fn run_command(options: RunOptions) -> io::Result<Output> {
    let shell_switches = match options.shell {
        Some(ref shell) => shell.get_switches(),
        None => Vec::new(),
    };

    if let Some(ref shell) = options.shell {
        let mut command = Command::new(shell.program_name());
        command.args(shell_switches);
        command.arg(options.program);
        if let Some(args) = options.args {
            command.arg(args.join(" "));
        }
        if let Some(cwd) = options.cwd {
            command.current_dir(&cwd);
        }
        command.output()
    } else {
        let mut command = Command::new(&options.program);
        if let Some(args) = options.args {
            command.args(args);
        }
        if let Some(cwd) = options.cwd {
            command.current_dir(&cwd);
        }
        command.output()
    }
}

fn process_run(luau: &Lua, run_options: LuaValue) -> LuaValueResult {
    let function_name = "process.run(options: RunOptions)";
    let options = match run_options {
        LuaValue::Table(run_options) => {
            RunOptions::from_table(luau, run_options)?
        },
        LuaValue::Nil => {
            return wrap_err!("{} expected RunOptions table of type {{ program: string, args: {{string}}?, shell: string?, cwd: string? }}, got nil.", function_name);
        },
        other => {
            return wrap_err!("{} expected RunOptions table of type {{ program: string, args: {{string}}?, shell: string?, cwd: string? }}, got: {:#?}", function_name, other);
        }
    };

    let program_to_run= options.program.clone();

    match run_command(options) {
        Ok(output) => {
            create_run_result_table(luau, output)
        },
        Err(err) => {
            // we want to throw an error if the program was unable to spawn at all
            // this is because when a user calls process.run/shell, they expect their program to actually run
            // and we don't want the 'ok' or 'err' value to serve two purposes (program failed to execute vs program executed with error)
            wrap_err!("{} was unable to run the program '{}': {}", function_name, program_to_run, err)
        }
    }
}

fn process_shell(luau: &Lua, shell_command: LuaValue) -> LuaValueResult {
    let function_name = "process.shell(command: string)";
    let shell_name = std_env::get_current_shell();
    let shell_command = match shell_command {
        LuaValue::String(command) => {
            command.to_str()?.to_string()
        },
        other => {
            return wrap_err!("{} expected command to be a string, got: {:?}", function_name, other);
        }
    };
    
    let run_options = RunOptions {
        program: shell_command.clone(),
        args: None,
        shell: Some(Shell::from(shell_name.clone())),
        cwd: None,
    };

    match run_command(run_options) {
        Ok(output) => create_run_result_table(luau, output),
        Err(err) => {
            wrap_err!("{} unable to run shell command '{}' with shell '{}' because of err: {}", function_name, shell_command, shell_name, err)
        }
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

    let shell_switches = match options.shell {
        Some(ref shell) => shell.get_switches(),
        None => Vec::new(),
    };

    let mut child = {
        match if let Some(ref shell) = options.shell {
            let mut command = Command::new(shell.program_name());
            command
                .args(shell_switches)
                .arg(options.program);
            if let Some(args) = options.args {
                command.arg(args.join(" "));
            }
            command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        } else {
            let mut command = Command::new(options.program);
            if let Some(args) = options.args {
               command.args(args);
            }
            command
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
    let arc_stdout = Arc::new(Mutex::new(Some(stdout)));
    let arc_stderr = Arc::new(Mutex::new(Some(stderr)));
    let arc_stdin = Arc::new(Mutex::new(stdin));

    /// makes it less annoying to get ChildStdout or ChildStderr from the arcs above
    #[inline(always)]
    fn unwrap_stream<T>(mutex: &Arc<Mutex<Option<T>>>, function_name: &'static str, stream_type: &'static str) -> LuaResult<T> {
        match mutex.lock().unwrap().take() {
            Some(stream) => Ok(stream),
            None => {
                wrap_err!("{}: unable to take ChildProcess' {}, are you trying to use multiple :read methods at the same time?", function_name, stream_type)
            }
        }
    }

    let stdout_handle = TableBuilder::create(luau)?
        .with_function("read", {
            let arc_stdout = Arc::clone(&arc_stdout);
            move | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                let function_name = "ChildProcess.stdout:read(count: number?, timeout: number?)";

                pop_self(&mut multivalue, function_name)?;
                let byte_count = match multivalue.pop_front() {
                    Some(LuaValue::Integer(i)) => i as usize,
                    Some(LuaValue::Number(f)) => f.trunc() as usize,
                    Some(LuaNil) | None => 1024,
                    Some(other) => {
                        return wrap_err!("{} expected count to be a number or nil, got: {:?}", function_name, other);
                    },
                };
                let timeout = match multivalue.pop_front() {
                    Some(LuaValue::Integer(i)) => i as f64,
                    Some(LuaValue::Number(f)) => f,
                    Some(LuaNil) | None => 0.015,
                    Some(other) => {
                        return wrap_err!("{} expected timeout to be a number or nil, got: {:?}", function_name, other);
                    },
                };

                let stdout = unwrap_stream(&arc_stdout, function_name, "stdout")?;
                let mut stream = Stream::new(stdout);
                
                let mut result: Vec<u8> = Vec::new();
                let mut last_read_time = Instant::now();

                loop {
                    let now = Instant::now();
                    let elapsed = (now - last_read_time).as_secs_f64();
                    if let Some(data) = stream.try_read() && elapsed <= timeout {
                        last_read_time = now;
                        if result.len() < byte_count {
                            result.push(data);
                        } else {
                            break;
                        }
                    } else if elapsed <= timeout {
                        continue;
                    } else {
                        break;
                    }
                }

                stream.recover(function_name, &arc_stdout)?;

                if result.is_empty() {
                    Ok(LuaNil)
                } else {
                    ok_string(result, luau)
                }
            }
        })?
        .with_function("read_until", {
            let arc_stdout = Arc::clone(&arc_stdout);
            move | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                let function_name = "ChildProcess.stdout:read_until(token: string, timeout: number?)";

                pop_self(&mut multivalue, function_name)?;
                let token = match multivalue.pop_front() {
                    Some(LuaValue::String(token)) => {
                        let token = token.as_bytes().to_owned();
                        if token.is_empty() {
                            return wrap_err!("{}: token must be a non-empty string", function_name);
                        } else {
                            token
                        }
                    },
                    Some(other) => {
                        return wrap_err!("{} expected token to be a string, got: {:?}", function_name, other);
                    },
                    None => {
                        return wrap_err!("{} expected token but was incorrectly called with zero arguments", function_name);
                    }
                };
                let timeout = match multivalue.pop_front() {
                    Some(LuaValue::Integer(i)) => Some(i as f64),
                    Some(LuaValue::Number(f)) => Some(f),
                    Some(LuaNil) | None => None,
                    Some(other) => {
                        return wrap_err!("{} expected timeout to be a number or nil, got: {:?}", function_name, other);
                    },
                };

                let result: Vec<u8> = if let Some(timeout) = timeout {
                    // rust currently doesn't provide an easy way for us to read from a stream and ensure it doesn't block
                    // so we need to throw the blocking reads in a different thread and use message passing
                    let stdout = unwrap_stream(&arc_stdout, function_name, "stdout")?;

                    let mut result = Vec::new();
                    let start_time = Instant::now();
                    let mut stream = Stream::new(stdout);

                    while let now = Instant::now() 
                        && let elapsed = now - start_time 
                        && elapsed.as_secs_f64() < timeout
                    {
                        if let Some(stuff) = stream.try_read() {
                            result.push(stuff);
                        }
                        if result.ends_with(&token) {
                            stream.kill(function_name)?;
                            break;
                        };
                    }

                    stream.recover(function_name, &arc_stdout)?;

                    result
                } else { // if the user didn't request a timeout we can do the easy thing and just keep reading til we see token
                    let mut stdout = match arc_stdout.lock().unwrap().take() {
                        Some(stdout) => stdout,
                        None => {
                            return wrap_err!("{}: unable to take ChildProcess' stdout, are you trying to use multiple :read methods at the same time?", function_name);
                        }
                    };

                    let mut result: Vec<u8> = Vec::new();
                    loop {
                        let mut buffer = [0u8; 1];
                        match stdout.read_exact(&mut buffer) {
                            Ok(_) => {
                                result.extend_from_slice(&buffer);
                                if result.ends_with(&token) {
                                    break;
                                }
                            },
                            Err(_err) => break
                        };
                    }

                    result
                };

                if result.is_empty() {
                    Ok(LuaNil)
                } else {
                    ok_string(result, luau)
                }
            }
        })?
        .with_function("read_exact", {
            let arc_stdout = Arc::clone(&arc_stdout);
            move | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                let function_name = "ChildProcess.stdout:read_exact(buffer_size: number)";
                let buffer_size = match multivalue.pop_back() {
                    Some(LuaValue::Integer(i)) => i as usize,
                    Some(LuaValue::Number(n)) => {
                        if n.trunc() == n {
                            n as usize
                        } else {
                            return wrap_err!("{} expected buffer_size to be an integer, got a float: {}", function_name, n);
                        }
                    },
                    _ => 32,
                };
                let mut stdout = unwrap_stream(&arc_stdout, function_name, "stdout")?;

                let mut buffy = vec![0; buffer_size];
                let res = match stdout.read_exact(&mut buffy) {
                    Ok(_) => {
                        let result_string = luau.create_string(buffy)?;
                        Ok(LuaValue::String(result_string))
                    }, // this method returns nil if EOF was reached before filling whole buffer
                    Err(_err) => Ok(LuaValue::Nil)
                };
                *arc_stdout.lock().unwrap() = Some(stdout); // give stdout backk so other :read methods can use
                res
            }
        })?
        .with_function("lines", {
            move | luau: &Lua, _multivalue: LuaMultiValue | -> LuaValueResult {
                let function_name = "ChildProcess.stdout:lines()";
                let arc_stdout = Arc::clone(&arc_stdout);
                let mut stdout = unwrap_stream(&arc_stdout, function_name, "stdout")?;
                Ok(LuaValue::Function(luau.create_function_mut({
                    move | luau: &Lua, _value: LuaValue | -> LuaValueResult {
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
            }
        })?
        .build_readonly()?;

    let stderr_handle = TableBuilder::create(luau)?
        .with_function("read_until", {
            let stderr = Arc::clone(&arc_stderr);
            move | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                let function_name = "ChildProcess.stderr:read_until(token: string, timeout: number?)";

                pop_self(&mut multivalue, function_name)?;
                let token = match multivalue.pop_front() {
                    Some(LuaValue::String(token)) => {
                        let token = token.as_bytes().to_owned();
                        if token.is_empty() {
                            return wrap_err!("{}: token must be a non-empty string", function_name);
                        } else {
                            token
                        }
                    },
                    Some(other) => {
                        return wrap_err!("{} expected token to be a string, got: {:?}", function_name, other);
                    },
                    None => {
                        return wrap_err!("{} expected token but was incorrectly called with zero arguments", function_name);
                    }
                };
                let timeout = match multivalue.pop_front() {
                    Some(LuaValue::Integer(i)) => Some(i as f64),
                    Some(LuaValue::Number(f)) => Some(f),
                    Some(LuaNil) | None => None,
                    Some(other) => {
                        return wrap_err!("{} expected timeout to be a number or nil, got: {:?}", function_name, other);
                    },
                };

                let result: Vec<u8> = if let Some(timeout) = timeout {
                    // rust currently doesn't provide an easy way for us to read from a stream and ensure it doesn't block
                    // so we need to throw the blocking reads in a different thread and use message passing
                    let stderr = unwrap_stream(&stderr, function_name, "stderr")?;

                    let mut result = Vec::new();
                    let start_time = Instant::now();
                    let mut stream = Stream::new(stderr);

                    while let now = Instant::now() 
                        && let elapsed = now - start_time 
                        && elapsed.as_secs_f64() < timeout
                    {
                        if let Some(stuff) = stream.try_read() {
                            result.push(stuff);
                        }
                        if result.ends_with(&token) {
                            stream.kill(function_name)?;
                            break;
                        };
                    }

                    result
                } else { // if the user didn't request a timeout we can do the easy thing and just keep reading til we see token
                    let mut stderr = match stderr.lock().unwrap().take() {
                        Some(stderr) => stderr,
                        None => {
                            return wrap_err!("{}: unable to take ChildProcess' stderr, are you trying to use multiple :read methods at the same time?", function_name);
                        }
                    };

                    let mut result: Vec<u8> = Vec::new();

                    loop {
                        let mut buffer = [0u8; 1];
                        match stderr.read_exact(&mut buffer) {
                            Ok(_) => {
                                result.extend_from_slice(&buffer);
                                if result.ends_with(&token) {
                                    break;
                                }
                            },
                            Err(_err) => break
                        };
                    }

                    result
                };

                if result.is_empty() {
                    Ok(LuaNil)
                } else {
                    ok_string(result, luau)
                }
            }
        })?
        .with_function("read_exact", {
            let stderr = Arc::clone(&arc_stderr);
            move | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                let function_name = "ChildProcess.stderr:read_exact(buffer_size: number)";
                let buffer_size = match multivalue.pop_back() {
                    Some(LuaValue::Integer(i)) => i as usize,
                    Some(LuaValue::Number(n)) => {
                        if n.trunc() == n {
                            n as usize
                        } else {
                            return wrap_err!("{} expected buffer_size to be an integer, got a float: {}", function_name, n);
                        }
                    },
                    _ => 32,
                };
                let mut stderr = unwrap_stream(&stderr, function_name, "stderr")?;
                let mut buffy = vec![0; buffer_size];
                match stderr.read_exact(&mut buffy) {
                    Ok(_) => {
                        let result_string = luau.create_string(buffy)?;
                        Ok(LuaValue::String(result_string))
                    }, // this method returns nil if EOF was reached before filling whole buffer
                    Err(_err) => Ok(LuaValue::Nil)
                }
            }
        })?
        .with_function("lines", {
            move | luau: &Lua, _multivalue: LuaMultiValue | -> LuaValueResult {
                let function_name = "ChildProcess.stderr:lines()";
                let stderr = Arc::clone(&arc_stderr);
                Ok(LuaValue::Function(luau.create_function({
                    move | luau: &Lua, _value: LuaValue | -> LuaValueResult {
                        let mut stderr = unwrap_stream(&stderr, function_name, "stderr")?;
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
                    Err(err) => {
                        if err.kind() == io::ErrorKind::BrokenPipe {
                            Ok(LuaNil)
                        } else {
                            wrap_err!("ChildProcess.stdin:write: error writing to stdin: {:?}", err)
                        }
                    }
                }
            }
        })?
        .build_readonly()?;
    
    let child_handle = TableBuilder::create(luau)?
        .with_value("id", child_id)?
        .with_function("alive", {
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

pub fn _handle_exit_callback(luau: &Lua, exit_code: i32) -> LuaResult<()> {
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
            f.call::<i64>(exit_code)?;
        },
        LuaValue::Nil => {},
        other => {
            unreachable!("wtf is in _process_exit_callback_function other than a function or nil?: {:?}", other)
        }
    }
    if let Ok(exit_code) = i32::try_from(exit_code) {
        process::exit(exit_code);
    } else {
        wrap_err!("process.exit: your exit code is too big ({}), we can't convert it to i32.", exit_code)
    }
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
