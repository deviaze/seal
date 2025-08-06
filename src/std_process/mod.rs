use core::str;
use std::cell::RefCell;
use std::fmt::Debug;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output, Stdio};
use std::rc::Rc;

use crate::prelude::*;
use crate::std_env;
use mluau::prelude::*;

mod stream;
use stream::{Stream, TruncateSide};

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
        let shell_name = Path::new(&s).file_name().and_then(|name| name.to_str()).unwrap_or(&s); // If file_name fails, fall back to the original

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

/// Represents process lib's `RunOptions` and `SpawnOptions` and I don't feel like making this an enum
#[derive(Debug)]
struct ProcessOptions {
    program: String,
    args: Option<Vec<String>>,
    shell: Option<Shell>,
    cwd: Option<PathBuf>,
    stdout_capacity: Option<usize>,
    stderr_capacity: Option<usize>,
    stdout_truncate: Option<TruncateSide>,
    stderr_truncate: Option<TruncateSide>,
}

impl ProcessOptions {
    fn from_table(luau: &Lua, run_options: LuaTable) -> LuaResult<Self> {
        let program = match run_options.raw_get("program")? {
            LuaValue::String(program) => program.to_string_lossy(),
            LuaValue::Nil => {
                return wrap_err!("SpawnOptions/RunOptions missing field `program`; expected string, got nil");
            }
            other => {
                return wrap_err!("SpawnOptions/RunOptions.program expected to be a string, got: {:#?}", other);
            }
        };

        let args = match run_options.raw_get("args")? {
            LuaValue::Table(args) => {
                let mut rust_vec: Vec<String> = Vec::from_lua(LuaValue::Table(args), luau)?;
                // let's trim the whitespace just to make sure we pass valid args (untrimmed args might explode)
                for s in rust_vec.iter_mut() {
                    *s = s.trim().to_string();
                }
                Some(rust_vec)
            }
            LuaValue::Nil => None,
            other => {
                return wrap_err!("SpawnOptions/RunOptions.args expected to be {{string}} or nil, got: {:#?}", other);
            }
        };

        let shell = match run_options.raw_get("shell")? {
            LuaValue::String(shell) => Some(Shell::from(shell.to_string_lossy())),
            LuaValue::Nil => None,
            other => {
                return wrap_err!("SpawnOptions/RunOptions.shell expected to be a string or nil, got: {:#?}", other);
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
                        return wrap_err!(
                            "SpawnOptions/RunOptions.cwd must be able to be canonicalized as an absolute path that currently exists on the filesystem; \
                        canonicalization failed with err: {}",
                            err
                        );
                    }
                };
                Some(canonicalized_cwd)
            }
            LuaNil => None,
            other => {
                return wrap_err!("SpawnOptions/RunOptions.cwd expected to be a string or nil, got: {:?}", other);
            }
        };

        let (stdout_capacity, stderr_capacity, stdout_truncate, stderr_truncate) = match run_options.raw_get("stream")? {
            LuaValue::Table(stream_table) => (
                match stream_table.raw_get("stdout_capacity")? {
                    LuaValue::Number(f) => float_to_usize(f, "SpawnOptions.capacity.stdout", "stdout")?,
                    LuaValue::Integer(i) => int_to_usize(i, "SpawnOptions.capacity.stdout", "stdout")?,
                    LuaNil => 2048_usize,
                    other => {
                        return wrap_err!("SpawnOptions.stream.stdout expected to be number or nil, got: {:?}", other);
                    }
                },
                match stream_table.raw_get("stderr_capacity")? {
                    LuaValue::Number(f) => float_to_usize(f, "SpawnOptions.capacity.stderr", "stderr")?,
                    LuaValue::Integer(i) => int_to_usize(i, "SpawnOptions.capacity.stderr", "stderr")?,
                    LuaNil => 1024_usize,
                    other => {
                        return wrap_err!("SpawnOptions.stream.stdout expected to be number or nil, got: {:?}", other);
                    }
                },
                match stream_table.raw_get("stdout_truncate")? {
                    LuaValue::String(t) => {
                        let t = match t.to_str() {
                            Ok(s) => s.to_string(),
                            Err(_) => {
                                return wrap_err!("SpawnOptions.stream.stdout_truncate must be \"front\" or \"back\", got an invalid utf-8 string (why)");
                            }
                        };
                        match t.as_str() {
                            "front" => TruncateSide::Front,
                            "back" => TruncateSide::Back,
                            other => {
                                return wrap_err!("SpawnOptions.stream.stdout_truncate expected \"front\" or \"back\" (default \"front\"), got: {}", other);
                            }
                        }
                    }
                    LuaNil => TruncateSide::Front,
                    other => {
                        return wrap_err!("SpawnOptions.stream.stdout_truncate expected to be \"front\" or \"back\", got: {:?}", other);
                    }
                },
                match stream_table.raw_get("stderr_truncate")? {
                    LuaValue::String(t) => {
                        let t = match t.to_str() {
                            Ok(s) => s.to_string(),
                            Err(_) => {
                                return wrap_err!("SpawnOptions.stream.stderr_truncate must be \"front\" or \"back\", got an invalid utf-8 string (why)");
                            }
                        };
                        match t.as_str() {
                            "front" => TruncateSide::Front,
                            "back" => TruncateSide::Back,
                            other => {
                                return wrap_err!("SpawnOptions.stream.stderr_truncate expected \"front\" or \"back\" (default \"front\"), got: {}", other);
                            }
                        }
                    }
                    LuaNil => TruncateSide::Front,
                    other => {
                        return wrap_err!("SpawnOptions.stream.stderr_truncate expected to be \"front\" or \"back\", got: {:?}", other);
                    }
                },
            ),
            LuaNil => (2048_usize, 1024_usize, TruncateSide::Front, TruncateSide::Front),
            other => {
                return wrap_err!("SpawnOptions.capacity expected to be a table or nil, got: {:?}", other);
            }
        };

        Ok(ProcessOptions {
            program,
            args,
            shell,
            cwd,
            stdout_capacity: Some(stdout_capacity),
            stderr_capacity: Some(stderr_capacity),
            stdout_truncate: Some(stdout_truncate),
            stderr_truncate: Some(stderr_truncate),
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
        }
    };

    let default_value: Option<LuaValue> = match multivalue.pop_front() {
        Some(LuaValue::String(default)) => Some(LuaValue::String(default)),
        Some(LuaValue::Function(f)) => Some(LuaValue::Function(f)),
        Some(LuaNil) => Some(LuaNil),
        Some(other) => {
            return wrap_err!("{}: expected default value to be a string (or a function that returns one), got: {:?}", function_name, other);
        }
        None => None,
    };

    let is_ok = match run_result.raw_get("ok")? {
        LuaValue::Boolean(b) => b,
        other => {
            return wrap_err!("{}: expected RunResult.ok to be a boolean, got: {:?}", function_name, other);
        }
    };

    let stdout = if is_ok {
        match run_result.raw_get("stdout")? {
            LuaValue::String(s) => {
                let Ok(s) = s.to_str() else {
                    return wrap_err!(
                        "{}: stdout is not a valid utf-8 encoded string, use RunResult.stdout to get the raw stdout without attempting to trim/clean it",
                        function_name
                    );
                };
                let s = s.trim_end();
                LuaValue::String(luau.create_string(s)?)
            }
            other => {
                return wrap_err!("{} RunResult.stdout is not a string??: {:?}", function_name, other);
            }
        }
    } else if let Some(default_value) = default_value {
        match default_value {
            LuaValue::String(d) => LuaValue::String(d),
            LuaValue::Function(f) => match f.call::<LuaValue>(run_result) {
                Ok(LuaValue::String(default)) => LuaValue::String(default),
                Ok(other) => {
                    return wrap_err!("{}: expected default value function to return string, got: {:?}", function_name, other);
                }
                Err(err) => {
                    return wrap_err!("{}: default value function unexpectedly errored: {}", function_name, err);
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
        Err(_) => vec,
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
            move |luau: &Lua, _value: LuaMultiValue| -> LuaValueResult {
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

fn run_command(options: ProcessOptions) -> io::Result<Output> {
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
        LuaValue::Table(run_options) => ProcessOptions::from_table(luau, run_options)?,
        LuaValue::Nil => {
            return wrap_err!(
                "{} expected RunOptions table of type {{ program: string, args: {{string}}?, shell: string?, cwd: string? }}, got nil.",
                function_name
            );
        }
        other => {
            return wrap_err!(
                "{} expected RunOptions table of type {{ program: string, args: {{string}}?, shell: string?, cwd: string? }}, got: {:#?}",
                function_name,
                other
            );
        }
    };

    let program_to_run = options.program.clone();

    match run_command(options) {
        Ok(output) => create_run_result_table(luau, output),
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
        LuaValue::String(command) => command.to_str()?.to_string(),
        other => {
            return wrap_err!("{} expected command to be a string, got: {:?}", function_name, other);
        }
    };

    let run_options = ProcessOptions {
        program: shell_command.clone(),
        args: None,
        shell: Some(Shell::from(shell_name.clone())),
        cwd: None,
        stdout_capacity: None,
        stderr_capacity: None,
        stdout_truncate: None,
        stderr_truncate: None,
    };

    match run_command(run_options) {
        Ok(output) => create_run_result_table(luau, output),
        Err(err) => {
            wrap_err!("{} unable to run shell command '{}' with shell '{}' because of err: {}", function_name, shell_command, shell_name, err)
        }
    }
}

fn process_spawn(luau: &Lua, spawn_options: LuaValue) -> LuaValueResult {
    let function_name = "process.spawn(options: SpawnOptions)";
    let options = match spawn_options {
        LuaValue::Table(run_options) => ProcessOptions::from_table(luau, run_options)?,
        LuaValue::Nil => {
            return wrap_err!(
                "{} expected a RunOptions table of type {{ program: string, args: {{string}}?, shell: string? }}, got nil",
                function_name
            );
        }
        other => {
            return wrap_err!(
                "{} expected RunOptions table of type {{ program: string, args: {{string}}?, shell: string? }}, got: {:#?}",
                function_name,
                other
            );
        }
    };

    let shell_switches = match options.shell {
        Some(ref shell) => shell.get_switches(),
        None => Vec::new(),
    };

    let mut child = {
        match if let Some(ref shell) = options.shell {
            let mut command = Command::new(shell.program_name());
            command.args(shell_switches).arg(options.program);
            if let Some(args) = options.args {
                command.arg(args.join(" "));
            }
            command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()
        } else {
            let mut command = Command::new(options.program);
            if let Some(args) = options.args {
                command.args(args);
            }
            command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()
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
    let mut stdin = child.stdin.take().unwrap();

    let child_cell = Rc::new(RefCell::new(child));

    let child_process_handle = {
        let stdout_stream = Stream::new(
            function_name,
            stdout,
            stream::StreamType::Stdout,
            options.stdout_capacity.unwrap_or(2048),
            options.stdout_truncate.unwrap_or(TruncateSide::Front),
        )?;
        let stdout_cell = Rc::new(RefCell::new(stdout_stream));
        let stdout_handle = Stream::create_handle(stdout_cell, luau)?;

        let stderr_stream = Stream::new(
            function_name,
            stderr,
            stream::StreamType::Stderr,
            options.stderr_capacity.unwrap_or(1024),
            options.stderr_truncate.unwrap_or(TruncateSide::Front),
        )?;
        let stderr_cell = Rc::new(RefCell::new(stderr_stream));
        let stderr_handle = Stream::create_handle(stderr_cell, luau)?;

        let stdin_handle = TableBuilder::create(luau)?
            .with_function_mut("write", {
                move |_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaEmptyResult {
                    let function_name = "child.stdin:write(data: string)";
                    pop_self(&mut multivalue, function_name)?;
                    let data_to_write = match multivalue.pop_front() {
                        Some(LuaValue::String(data)) => data.as_bytes().to_vec(),
                        Some(LuaValue::Buffer(b)) => b.to_vec(),
                        Some(other) => {
                            return wrap_err!("{} expected data to be a string or buffer, got: {:?}", function_name, other);
                        }
                        None => {
                            return wrap_err!("{} expected data to be string or buffer, unexpectedly got nothing (not even nil)", function_name);
                        }
                    };

                    match stdin.write_all(&data_to_write) {
                        Ok(_) => Ok(()),
                        Err(err) => {
                            wrap_err!("{} can't write to stdin due to err: {}", function_name, err)
                        }
                    }
                }
            })?
            .build_readonly()?;

        TableBuilder::create(luau)?
            .with_value("id", child_id)?
            .with_value("stdout", stdout_handle)?
            .with_value("stderr", stderr_handle)?
            .with_value("stdin", stdin_handle)?
            .with_function("alive", {
                let child_cell = Rc::clone(&child_cell);
                move |_luau: &Lua, _value: LuaValue| -> LuaValueResult {
                    let function_name = "ChildProcess:alive()";
                    match child_cell.try_borrow_mut() {
                        Ok(ref mut child) => match child.try_wait().unwrap() {
                            Some(_status_code) => Ok(LuaValue::Boolean(false)),
                            None => Ok(LuaValue::Boolean(true)),
                        },
                        Err(_) => {
                            wrap_err!("{}: child already borrowed", function_name)
                        }
                    }
                }
            })?
            .with_function("kill", {
                let function_name = "ChildProcess:kill()";
                let child_cell = Rc::clone(&child_cell);
                move |_luau: &Lua, _value: LuaValue| -> LuaEmptyResult {
                    match child_cell.try_borrow_mut() {
                        Ok(ref mut child) => match child.kill() {
                            Ok(_) => Ok(()),
                            Err(err) => {
                                wrap_err!("{} could not murder child due to err: {}", function_name, err)
                            }
                        },
                        Err(_) => {
                            wrap_err!("{}: child already borrowed", function_name)
                        }
                    }
                }
            })?
            .build_readonly()
    };

    ok_table(child_process_handle)
}

fn set_exit_callback(luau: &Lua, f: Option<LuaValue>) -> LuaValueResult {
    if let Some(f) = f {
        match f {
            LuaValue::Function(f) => {
                let globals = luau.globals();
                globals.set("_process_exit_callback_function", f)?;
                Ok(LuaNil)
            }
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
        }
        LuaValue::Nil => {}
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
        }
        LuaValue::Nil => {}
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
