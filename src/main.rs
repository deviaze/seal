#![feature(default_field_values)]
use crate::{prelude::*, setup::SetupOptions};
use mluau::prelude::*;

use std::env;
use std::ffi::OsString;
use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::Write;

pub mod prelude;
mod std_env;
mod std_fs;
mod std_json;
mod std_process;
mod std_time;
pub mod table_helpers;
#[macro_use]
mod err;
mod globals;
mod interop;
mod require;
mod std_crypt;
mod std_io;
mod std_net;
mod std_serde;
mod std_str_internal;
mod std_thread;
mod std_luau;
mod std_err;
mod sealconfig;
mod setup;
mod compile;
mod std_args;

use err::display_error_and_exit;
use sealconfig::SealConfig;
use globals::SEAL_VERSION;

type LuauLoadResult = LuaResult<Option<LuauLoadInfo>>;
struct LuauLoadInfo {
    luau: Lua,
    src: Vec<u8>,
    /// chunk_name is basically the entry_path except it's always an absolute path
    chunk_name: String,
}

type Args = VecDeque<OsString>;
#[derive(Debug)]
enum SealCommand {
    /**
    Runs `seal` with a valid luau module path/filename (must be `*.luau` or directory w/ `init.luau`)

    ## Examples:
    * `seal ./hi.luau`
    * `seal ./hi.luau meow1 meow2`
    */
    Default { filename: String },
    /** 
    Evaluate some string `src` with `seal`; `fs`, `http`, and `process` libs are already loaded in for convenience.
    
    ## Examples:
    * `seal eval 'print("hi")'`
    * `seal eval 'print(process.shell({ program = "seal -h" }):unwrap())'` 
    */ 
    Eval(Args),
    /** 
    Run `seal` at the project (at your cwd)'s entrypoint, usually `./src/main.luau` unless configured otherwise.
    
    ## Examples:
    * `seal run arg1 arg2`
    */ 
    Run,
    /// `seal setup` | `seal project` | `seal script` | `seal setup custom <args>`
    Setup(SetupOptions),
    /// Display `seal` help.
    DefaultHelp,
    CommandHelp(Box<SealCommand>),
    HelpCommandHelp,
    SealConfigHelp,
    /// `seal test` (runs test_path from config.luau)
    Test,
    Version,
    /// not yet implemented
    Repl,
    ExecStandalone(Vec<u8>),
    /// Compiles project codebase to standalone executable (or bundles to a .luau file)
    /// seal compile ./myfile.luau sets ./myfile.luau as the entry point file (otherwise defaults to .seal/config.luau entry_path)
    /// seal compile [path.luau] -o binname names the output executable 'binname'
    /// seal compile [path.luau] -o filename.luau bundles the project's sourcecode into filename.luau without making a standalone executable
    Compile(Args),
}

impl SealCommand {
    fn from(s: &str, args: Args) -> LuaResult<Self> {
        Ok(match s {
            "version" | "--version" | "-V" => Self::Version,
            "setup" | "s" => Self::Setup(SetupOptions::from_args(&args)?),
            "project" | "sp" => Self::Setup(SetupOptions::Project),
            "script" | "ss" => Self::Setup(SetupOptions::Script),
            "custom" | "sc" => Self::Setup(SetupOptions::Custom),
            "compile" => Self::Compile(args),
            "eval" | "e" => Self::Eval(args.clone()),
            "run" | "r" => Self::Run,
            "test" | "t" => Self::Test,
            "repl" | "i" => Self::Repl,
            "help" | "h" => Self::figure_out_which_command_we_need_help_with(args)?,
            // default case `seal ./myfile.luau`
            filename => Self::Default { filename: filename.to_owned() },
        })
    }
    // rest of the SealCommand impl defined at the bottom of main.rs
}


fn main() -> LuaResult<()> {
    // We intercept SIGABRT on *nix to prevent core dumps when seal is used as a child process
    err::setup_sigabrt_handler();
    err::setup_panic_hook(); // seal panic = seal bug; we shouldn't panic in normal operation

    let args: VecDeque<OsString> = env::args_os().collect();

    let command = match SealCommand::parse(args) {
        Ok(command) => command,
        Err(err) => display_error_and_exit(err),
    };

    let info_result = match command {
        SealCommand::Default { filename } => {
            resolve_file(filename, "seal")
        },
        SealCommand::Eval(args) => seal_eval(args),
        SealCommand::Run => seal_run(),
        SealCommand::Setup(options) => seal_setup(options),
        SealCommand::Test => seal_test(),
        SealCommand::Version => {
            println!("{}", SEAL_VERSION);
            Ok(None)
        },
        SealCommand::CommandHelp(command) => command.help(),
        help @ SealCommand::DefaultHelp | 
        help @ SealCommand::HelpCommandHelp |
        help @ SealCommand::SealConfigHelp => help.help(),
        SealCommand::Repl => {
            wrap_err!("seal repl coming SOON (tm)")
        },
        SealCommand::Compile(args) => seal_compile(args),
        SealCommand::ExecStandalone(bytecode) => seal_standalone(bytecode),
    };

    let LuauLoadInfo { luau, src, chunk_name } = match info_result {
        Ok(Some(info)) => info,
        Ok(None) => return Ok(()),
        Err(err) => display_error_and_exit(err),
    };

    match luau.load(src).set_name(chunk_name).exec() {
        Ok(_) => Ok(()),
        Err(err) => display_error_and_exit(err),
    }
}

fn resolve_file(requested_path: String, function_name: &'static str) -> LuauLoadResult {
    if requested_path.ends_with(".lua") {
        return wrap_err!("{}: wrong language! seal only runs .luau files", function_name);
    }
    let Some(chunk_name) = require::get_chunk_name_for_module(&requested_path, function_name)? else {
        return wrap_err!("'{}' not found; does it exist and is it either a .luau file or directory with an init.luau?", requested_path);
    };
    
    let luau = Lua::default();
    if let Err(err) = luau.sandbox(true) {
        return wrap_err!("{}: unable to enable Luau safeenv (sandbox mode) on chunk '{}' due to err: {}", function_name, chunk_name, err);
    };

    globals::set_globals(&luau, chunk_name.clone())?;

    let mut src = match fs::read_to_string(&chunk_name) {
        Ok(src) => src,
        Err(err) => {
            return wrap_err!("{}: unable to read file at '{}' due to err: {}", function_name, chunk_name, err);
        }
    };

    // handle shebangs by stripping first line from \n
    if src.starts_with("#!") && let Some(first_newline_pos) = src.find('\n') {
        src = src[first_newline_pos + 1..].to_string();
    }

    Ok(Some(LuauLoadInfo { luau, src: src.as_bytes().to_owned(), chunk_name }))
}

fn seal_eval(mut args: Args) -> LuauLoadResult {
    let Some(os_src) = args.pop_front() else {
        return wrap_err!("seal eval got nothing to eval, did you forget to pass me the src?");
    };
    let Ok(src) = os_src.into_string() else {
        return wrap_err!("seal eval: luau code must be valid utf-8");
    };

    let luau = Lua::default();
    let globals = luau.globals();
    globals::set_globals(&luau, String::from("eval"))?;
    
    // eval comes with a few libs builtin
    globals.raw_set("fs", ok_table(std_fs::create(&luau))?)?;
    globals.raw_set("process", ok_table(std_process::create(&luau))?)?;
    globals.raw_set("http", ok_table(std_net::http::create(&luau))?)?;

    Ok(Some(LuauLoadInfo {
        luau,
        src: src.as_bytes().to_owned(),
        // relative require probs wont work atm
        chunk_name: std_env::get_cwd("seal eval")?
            .to_string_lossy()
            .into_owned(),
    }))
}

/// seal run basically just tries to run the entrypoint of the codebase if present
/// defaulting to ./src/main.luau and optionally specified/overriden in a .seal/config.luau
fn seal_run() -> LuauLoadResult {
    let function_name = "seal run";
    let luau = Lua::default();
    let entry_path = match SealConfig::read(&luau, None, function_name)? {
        Some(config) => config.entry_path,
        None => {
            return wrap_err!("{}: project missing .seal/config.luau and src/main.luau (default entry_path); \
            use seal ./filename.luau to run a specific file", function_name);
        },
    };
    globals::set_globals(&luau, entry_path.clone())?;
    resolve_file(entry_path, function_name)
}

fn seal_test() -> LuauLoadResult {
    let function_name = "seal test";
    let luau = Lua::default();
    let test_path = match SealConfig::read(&luau, None, function_name)? {
        Some(config) => config.test_path,
        None => {
            return wrap_err!("{}: no .seal/config.luau located upwards of your cwd; \
            use seal ./filename.luau to run a specific file", function_name);
        },
    };
    if let Some(test_path) = test_path {
        globals::set_globals(&luau, test_path.clone())?;
        resolve_file(test_path, function_name)
    } else {
        wrap_err!("{}: attempt to test a project without a 'test_path' field set in .seal/config.luau", function_name)
    }
}

fn seal_setup(options: SetupOptions) -> LuauLoadResult {
    setup::run(options)?;
    Ok(None)
}

fn seal_compile(mut args: Args) -> LuauLoadResult {
    let function_name = "seal compile";

    let default_entry_path = std_env::get_cwd(function_name)?;
    let default_output_path = match default_entry_path.file_name() {
        Some(basename) => basename.to_string_lossy(),
        None => {
            return wrap_err!("{} - why can't we figure out the basename of your cwd???", function_name);
        }
    };

    // ugly asf we should probably be parsing these in SealCommand
    #[allow(unused_mut, reason = "needs to be mut on windows")]
    let (entry_path, mut output_path): (String, String) = {
        if args.is_empty() {
            (default_entry_path.to_string_lossy().to_string(), default_output_path.to_string())
        } else if let Some(front) = args.front()
            && let Some(front) = front.to_str()
        {
            if front == "-o" {
                let _ = args.pop_front();
                if let Some(exec_name) = args.front() {
                    (default_entry_path.to_string_lossy().to_string(), exec_name.to_string_lossy().to_string())
                } else {
                    return wrap_err!("{} - output switch (-o) provided but missing output file name/path", function_name);
                }
            } else {
                let entry_path = args.pop_front().unwrap(); // UNWRAP: args never empty here
                if let Some(front) = args.front()
                    && let Some(front) = front.to_str()
                {
                    if front == "-o" {
                        let _ = args.pop_front();
                        if let Some(exec_name) = args.front() {
                            (entry_path.to_string_lossy().to_string(), exec_name.to_string_lossy().to_string())
                        } else {
                            return wrap_err!("{} - output switch (-o) provided but missing output file name/path", function_name);
                        }
                    } else {
                        (entry_path.to_string_lossy().to_string(), default_output_path.to_string())
                    }
                } else {
                    return wrap_err!("{} - bad utf8 :skull:", function_name);
                }

            }
        } else {
            return wrap_err!("{} - bad utf8 :skull:", function_name);
        }
    };

    let bundled_src = compile::bundle(&entry_path)?;

    if output_path.ends_with(".luau") {
        match fs::write(&output_path, bundled_src) {
            Ok(_) => {
                println!("{} - bundled project sourcecode to '{}'", function_name, &output_path);
            },
            Err(err) => {
                return wrap_err!("{} - unable to write to file '{}' due to err: {}", function_name, &output_path, err);
            }
        }
        return Ok(None);
    };

    let compiled_standalone_bytes = compile::standalone(&bundled_src)?;

    let mut file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&*output_path)
    {
        Ok(f) => f,
        Err(err) => {
            return wrap_err!("{} - error creating output file: {}", function_name, err);
        }
    };

    #[cfg(windows)]
    {
        if !output_path.ends_with(".exe") {
            output_path.push_str(".exe");
        }
    }


    if let Err(err) = file.write_all(&compiled_standalone_bytes) {
        return wrap_err!("{} - error writing compiled program to file: {}", function_name, err);
    }

    println!("{} - compiled to standalone program '{}'!", function_name, output_path);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // give it executable permissions on unix (chmod +x file) rwxr-xr-x (0o755)
        if let Err(err) = file.set_permissions(PermissionsExt::from_mode(0o755)) {
            return wrap_err!("{} - error setting executable permissions: {}", function_name, err);
        }
    }

    Ok(None)
}

fn seal_standalone(bytecode: Vec<u8>) -> LuauLoadResult {
    let luau = Lua::new();
    let entry_path = std::env::current_exe().unwrap_or_default();
    let entry_path = entry_path.to_string_lossy().into_owned();
    globals::set_globals(&luau, &entry_path)?;
    Ok(Some(LuauLoadInfo {
        luau,
        src: bytecode,
        chunk_name: entry_path
    }))
}

impl SealCommand {
    fn parse(mut args: Args) -> LuaResult<SealCommand> {
        if let Some(bytecode) = compile::extract_bytecode(None) {
            return Ok(Self::ExecStandalone(bytecode))
        }

        // discard first arg (always "seal")
        let _ = args.pop_front();

        // show help if user runs seal w/out anything else
        let Some(first_arg) = args.pop_front() else {
            eprintln!("seal: you didn't pass me anything :(\n  (expected file to run or command, displaying help)");
            return Ok(Self::DefaultHelp);
        };

        // command/filename should be utf-8
        let Some(first_arg) = first_arg.to_str() else {
            return wrap_err!("seal: filename/command not valid utf-8");
        };

        if first_arg == "--help" || first_arg == "-h" {
            return Ok(Self::DefaultHelp)
        }

        let command = Self::from(first_arg, args.clone())?;
        // `seal ./mycli.luau --help` should be passed to ./mycli.luau not directly to seal
        if !command.is_default() && command.next_is_help(&args) {
            Ok(Self::CommandHelp(Box::new(command)))
        } else {
            Ok(command)
        }
    }
    fn is_default(&self) -> bool {
        matches!(self, Self::Default { .. })
    }
    fn help(&self) -> LuauLoadResult {
        let luau_to_run_help = Lua::default();
        globals::set_globals(&luau_to_run_help, String::from("seal help"))?;
        let help_src = include_str!("./scripts/seal_help.luau");
        let help_table = match luau_to_run_help.load(help_src).eval() {
            Ok(LuaValue::Table(t)) => t,
            Ok(other) => {
                panic!("what did seal help return other than the help table?? (got {:?})", other);
            },
            Err(err) => {
                panic!("seal help errored at runtime: {}", err);
            }
        };
        let help_function: LuaFunction = help_table.raw_get::<LuaFunction>(match self {
            Self::Default {..} | Self::DefaultHelp => "default",
            Self::Eval(_) => "eval",
            Self::Run => "run",
            Self::Setup(_) => "setup",
            Self::Test => "test",
            Self::HelpCommandHelp => "help",
            Self::SealConfigHelp => "config",
            Self::Compile(_) => "compile",
            other => {
                return wrap_err!("help not yet implemented for command {:#?}", other);
            },
        })?;
        println!("{}", help_function.call::<String>(LuaNil)?);
        Ok(None)
    }
    fn next_is_help(&self, args: &Args) -> bool {
        if let Some(next) = args.front() && let Some(arg) = next.to_str() {
            matches!(arg, "-h" | "--help")
        } else {
            false
        }
    }
    fn figure_out_which_command_we_need_help_with(mut args: Args) -> LuaResult<SealCommand> {
        Ok(if let Some(arg) = args.pop_front() && let Ok(arg) = arg.into_string() {
            if arg == "config" {
                Self::SealConfigHelp
            } else if arg == "help" || arg == "h" {
                // `seal help help` or `seal help h`
                Self::HelpCommandHelp
            } else {
                Self::CommandHelp(Box::new(Self::from(&arg, args)?))
            }
        } else {
            Self::DefaultHelp
        })
    }
}