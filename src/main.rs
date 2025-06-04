#![feature(default_field_values)]
use crate::prelude::*;
use mlua::prelude::*;
use include_dir::{include_dir, Dir};

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
// mod std_serde_old;
mod std_serde;
mod std_str_internal;
mod std_testing;
mod std_thread;
mod sealconfig;

use err::display_error_and_exit;
use sealconfig::SealConfig;
use globals::SEAL_VERSION;

type LuauLoadResult = LuaResult<Option<LuauLoadInfo>>;
struct LuauLoadInfo {
    luau: Lua,
    src: String,
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
    /// Set up a new project for `seal`, spawning in a `.vscode`, `.luaurc`, `./src/main.luau` etc.
    Setup,
    /// Display `seal` help.
    DefaultHelp,
    CommandHelp(Box<SealCommand>),
    HelpCommandHelp,
    SealConfigHelp,
    /// `seal test` (runs test_path from sealconfig.luau)
    Test,
    Version,
    /// not yet implemented
    Repl,
}

impl SealCommand {
    fn from(s: &str, args: Args) -> Self {
        match s {
            "version" | "--version" | "-V" => Self::Version,
            "setup" | "s" => Self::Setup,
            "eval" | "e" => Self::Eval(args.clone()),
            "run" | "r" => Self::Run,
            "test" | "t" => Self::Test,
            "repl" | "i" => Self::Repl,
            "help" | "h" => Self::figure_out_which_command_we_need_help_with(args),
            // default case `seal ./myfile.luau`
            filename => Self::Default { filename: filename.to_owned() },
        }
    }
    // rest of the SealCommand impl defined at the bottom of main.rs
}

fn main() -> LuaResult<()> {
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
        SealCommand::Setup => seal_setup(),
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

    Ok(Some(LuauLoadInfo { luau, src, chunk_name }))
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
        src,
        // relative require probs wont work atm
        chunk_name: std_env::get_cwd("seal eval")?
            .to_string_lossy()
            .into_owned(),
    }))
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
        wrap_err!("{}: attempt to test a project without a 'test_path' field set in .seal/sealconfig.luau", function_name)
    }
}

fn seal_setup() -> LuauLoadResult {
    const DOT_SEAL_DIR: Dir = include_dir!("./.seal");
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

impl SealCommand {
    fn parse(mut args: Args) -> LuaResult<SealCommand> {
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

        let command = Self::from(first_arg, args.clone());
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
            Self::Setup => "setup",
            Self::Test => "test",
            Self::HelpCommandHelp => "help",
            Self::SealConfigHelp => "config",
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
    fn figure_out_which_command_we_need_help_with(mut args: Args) -> SealCommand {
        if let Some(arg) = args.pop_front() && let Ok(arg) = arg.into_string() {
            if arg == "config" {
                Self::SealConfigHelp
            } else if arg == "help" || arg == "h" {
                // `seal help help` or `seal help h`
                Self::HelpCommandHelp
            } else {
                Self::CommandHelp(Box::new(Self::from(&arg, args)))
            }
        } else {
            Self::DefaultHelp
        }
    }
}