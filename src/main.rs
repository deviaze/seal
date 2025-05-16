use crate::prelude::*;
use mlua::prelude::*;

use std::{collections::VecDeque, env, ffi::OsString, fs, io};

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
mod std_testing;
mod std_thread;
mod sealconfig;

use err::display_error_and_exit;
use sealconfig::SealConfig;
use globals::SEAL_VERSION;

use include_dir::{include_dir, Dir};
const DOT_SEAL_DIR: Dir = include_dir!("./.seal");

type LuauLoadResult = LuaResult<Option<LuauLoadInfo>>;
struct LuauLoadInfo {
    luau: Lua,
    src: String,
    /// chunk_name is basically the entry_path except it's always an absolute path
    chunk_name: String,
}

type Args = VecDeque<OsString>;
enum SealCommand {
    /**
    Runs `seal` with a valid luau module path/filename (must be `*.luau` or directory w/ `init.luau`)

    ## Examples:
    * `seal ./hi.luau`
    * `seal ./hi.luau meow1 meow2`
    */
    Default,
    /** 
    Evaluate some string `src` with `seal`; `fs`, `http`, and `process` libs are already loaded in for convenience.
    
    ## Examples:
    * `seal eval 'print("hi")'`
    * `seal eval 'print(process.shell({ program = "seal -h" }):unwrap())'` 
    */ 
    Eval,
    /** 
    Run `seal` at the project (at your cwd)'s entrypoint, usually `./src/main.luau` unless configured otherwise.
    
    ## Examples:
    * `seal run arg1 arg2`
    */ 
    Run,
    /// Set up a new project for `seal`, spawning in a `.vscode`, `.luaurc`, `./src/main.luau` etc.
    Setup,
    /// Display `seal` help.
    Help,
    Test,
}

impl SealCommand {
    fn load(mut args: Args) -> LuauLoadResult {
        // discard first arg "seal"
        let _ = args.pop_front();

        let Some(first_arg) = args.pop_front() else {
            eprintln!("seal: you didn't pass me anything :( expected a file or command, displaying help:\n");
            return seal_help(Self::Default);
        };

        let Some(first_arg) = first_arg.to_str() else {
            return wrap_err!("seal: first argument not valid utf-8");
        };

        match first_arg {
            "help" | "--help" | "-h" => {
                help_or(&args, Self::Help, || seal_help(Self::Default))
            },
            "version" | "--version" | "-v" => {
                println!("seal {SEAL_VERSION}");
                Ok(None)
            },
            "setup" | "s" => {
                help_or(&args, Self::Setup, seal_setup)
            },
            "test" => {
                help_or(&args, Self::Test, seal_test)
            },
            "eval" | "e" => {
                if next_is_help(&args) {
                    seal_help(Self::Eval)
                } else {
                    match args.pop_front() {
                        Some(s) => if let Ok(src) = s.into_string() {
                            seal_eval(src)
                        } else {
                            wrap_err!("seal eval: luau code must be valid utf-8")
                        },
                        None => {
                            wrap_err!("seal eval got nothing to eval, did you forget to pass me the src?")
                        }
                    }
                }
            },
            "repl" => wrap_err!("seal repl coming SOON (TM)"),
            "run" | "r" => help_or(&args, Self::Run, seal_run),
            other => resolve_file(other.to_string(), "seal"),
        }
    }
}

fn main() -> LuaResult<()> {
    err::setup_panic_hook(); // seal panic = seal bug; we shouldn't panic in normal operation

    let args: VecDeque<OsString> = env::args_os().collect();

    let LuauLoadInfo { luau, src, chunk_name } = match SealCommand::load(args) {
        Ok(Some(info)) => info,
        Ok(None) => {
            return Ok(());
        },
        Err(err) => display_error_and_exit(err),
    };

    match luau.load(src).set_name(chunk_name).exec() {
        Ok(_) => Ok(()),
        Err(err) => display_error_and_exit(err),
    }
}

fn resolve_file(requested_path: String, function_name: &'static str) -> LuauLoadResult {
    if let Some(chunk_name) = require::get_chunk_name_for_module(&requested_path, function_name)? {
        let luau = Lua::default();
        globals::set_globals(&luau, chunk_name.clone())?;
        let mut src = if chunk_name.ends_with(".luau") {
            match fs::read_to_string(&chunk_name) {
                Ok(src) => src,
                Err(err) => {
                    return wrap_err!("{}: unable to read file '{}': {}", function_name, chunk_name, err);
                }
            }
        } else {
            return wrap_err!("{}: wrong language! seal only runs .luau files", function_name)
        };
        // handle shebangs by stripping first line by slicing from first \n
        if src.starts_with("#!") && let Some(first_newline_pos) = src.find('\n') {
            src = src[first_newline_pos + 1..].to_string();
        }
        Ok(Some(LuauLoadInfo { luau, src, chunk_name }))
    } else {
        wrap_err!("'{}' not found; does it exist, and is it either a .luau file or a directory with an init.luau?", requested_path)
    }
}

/// seal run basically just tries to run the entrypoint of the codebase if present
/// defaulting to ./src/main.luau and optionally specified/overriden in a .seal/sealconfig.luau
fn seal_run() -> LuauLoadResult {
    let function_name = "seal run";
    let luau = Lua::default();
    let entry_path = match SealConfig::read(&luau, None, function_name)? {
        Some(config) => config.entry_path,
        None => {
            return wrap_err!("{}: no .seal/sealconfig.luau located upwards of your cwd; \
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
            return wrap_err!("{}: no .seal/sealconfig.luau located upwards of your cwd; \
            use seal ./filename.luau to run a specific file", function_name);
        },
    };
    if let Some(test_path) = test_path {
        globals::set_globals(&luau, test_path.clone())?;
        resolve_file(test_path, function_name)
    } else {
        wrap_err!("{}: attempt to test a project without a sealconfig 'test_path' field", function_name)
    }
}

fn seal_eval(src: String) -> LuauLoadResult {
    let luau = Lua::default();
    let globals = luau.globals();
    globals::set_globals(&luau, String::from("eval"))?;
    globals.raw_set("fs", ok_table(std_fs::create(&luau))?)?;
    globals.raw_set("process", ok_table(std_process::create(&luau))?)?;
    globals.raw_set("http", ok_table(std_net::http::create(&luau))?)?;

    Ok(Some(LuauLoadInfo {
        luau,
        src,
        // relative require probs wont work atm
        chunk_name: std_env::get_cwd("seal eval")?
            .to_string_lossy()
            .into_owned(),
    }))
}

fn seal_help(command: SealCommand) -> LuauLoadResult {
    let help_message = match command {
        SealCommand::Default => {
            "seal, the cutest runtime for luau\n  \
              Usage:\n    \
                `seal setup` - set up a project in an existing folder (at your cwd) with typedefs, config, etc.\n    \
                `seal ./filename.luau <arg1> <arg2>` - run a luau file with seal.\n    \
                `seal run <arg1> <arg2>` - run the current project at your cwd.\n    \
                `seal eval '<string src>'` - evaluate a string with seal with fs, process, and http loaded in.\n\n\
            Proper seal help will be implemented SOON(TM)."
        },
        SealCommand::Eval => {
            "Usage: seal eval '<src>'"
        },
        SealCommand::Run => {
            "Usage: seal run <arg1> <arg2>\n  \
              Runs the project at your current directory at its entrypoint.\n  \
              By default, a project's entry_path is './src/main.luau' but you can configure it in your '.seal/sealconfig.luau'"
        },
        SealCommand::Test => {
            "Usage: seal test\n  \
              Runs a luau file at the sealconfig test_path specified in .seal/sealconfig.luau"
        }
        SealCommand::Setup => {
            "Usage: seal setup\n  \
              Initializes a new project for seal in your current directory, setting up .seal, type definitions, \
              .vscode/settings.json, etc."
        },
        SealCommand::Help => "seal help --help help me",
    };
    println!("{help_message}");
    Ok(None)
}

fn seal_setup() -> LuauLoadResult {
    let cwd = std_env::get_cwd("seal setup")?;
    let dot_seal_dir = cwd.join(".seal");
    let created_seal_dir = match fs::create_dir(&dot_seal_dir) {
        Ok(_) => true,
        Err(err) => match err.kind() {
            io::ErrorKind::AlreadyExists => {
                println!("seal setup - '.seal' already exists; not replacing it");
                false
            }
            _ => {
                return wrap_err!("seal setup = error creating .seal: {}", err);
            }
        },
    };

    if created_seal_dir {
        match DOT_SEAL_DIR.extract(dot_seal_dir) {
            Ok(()) => {
                println!("seal setup .seal in your current directory!");
            }
            Err(err) => {
                return wrap_err!("seal setup - error extracting .seal directory: {}", err);
            }
        };
    }

    let seal_setup_settings = include_str!("./scripts/seal_setup_settings.luau");
    let temp_luau = Lua::new();
    globals::set_globals(&temp_luau, cwd.to_string_lossy().into_owned())?;
    match temp_luau.load(seal_setup_settings).exec() {
        Ok(_) => Ok(None),
        Err(err) => {
            wrap_err!("Hit an error running seal_setup_settings.luau: {}", err)
        }
    }
}

fn help_or<F>(args: &Args, command: SealCommand, run: F) -> LuauLoadResult
where F: FnOnce() -> LuauLoadResult
{
    if next_is_help(args) {
        seal_help(command)
    } else {
        run()
    }
}

fn next_is_help(args: &Args) -> bool {
    if let Some(next) = args.front()
    && let Some(arg) = next.to_str() {
        matches!(arg, "-h" | "--help")
    } else {
        false
    }
}