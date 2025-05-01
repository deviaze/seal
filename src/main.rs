use mlua::prelude::*;
use table_helpers::TableBuilder;
use std::{fs, env, panic, path::Path};
use std::io;

mod table_helpers;
mod std_io_output;
mod std_fs;
mod std_process;
mod std_env;
mod std_json;
mod std_time;
#[macro_use]
mod error_handling;
mod std_io;
mod std_io_colors;
mod std_io_input;
mod std_net;
mod std_net_http;
mod std_net_serve;
mod std_thread;
mod std_serde;
mod std_crypt;
mod std_testing;
mod globals;
mod require;
mod interop;

use crate::std_io_colors as colors;

use include_dir::{include_dir, Dir};
const TYPEDEFS_DIR: Dir = include_dir!(".typedefs");

type LuaValueResult = LuaResult<LuaValue>;
type LuaEmptyResult = LuaResult<()>;
type LuaMultiResult = LuaResult<LuaMultiValue>;

// wraps returns of stdlib::create functions with Ok(LuaValue::Table(t))
pub fn ok_table(t: LuaResult<LuaTable>) -> LuaValueResult {
    Ok(LuaValue::Table(t?))
}

pub fn ok_function(f: fn(&Lua, LuaValue) -> LuaValueResult, luau: &Lua) -> LuaValueResult {
    Ok(LuaValue::Function(luau.create_function(f)?))
}

pub fn ok_string<S: AsRef<[u8]>>(s: S, luau: &Lua) -> LuaValueResult {
    Ok(LuaValue::String(luau.create_string(s)?))
}

fn main() -> LuaResult<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 3 && args[2] == "--debug" {
        // don't mess with panic formatting
    } else {
        panic::set_hook(Box::new(|info| {
            let payload = info.payload().downcast_ref::<&str>().map(|s| s.to_string())
                .or_else(|| info.payload().downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "Unknown error running the custom panic hook, please report this to the manager (deviaze)".to_string());
            
            eprintln!("{}[ERR]{}{} {}{}", colors::BOLD_RED, colors::RESET, colors::RED, payload, colors::RESET);
        }));
    }

    if args.len() <= 1 {
        panic!("seal: did you forget to pass me a file?")
    }

    let first_arg = args[1].clone();

    if first_arg == "--help" || first_arg == "-h" {
        println!("seal help will be implemented SOON(TM)");
        return Ok(());
    }
    
    if first_arg == "setup" {
        return seal_setup();
    }
    
    let luau: Lua = Lua::new();
    
    // luau.sandbox(true)?; // free performance boost

    let globals = luau.globals();
    let mut entry_path = String::from("");

    let mut luau_code: String = {
        if first_arg == "eval" {
            let table = LuaValue::Table;

            let globals = luau.globals();
            globals.set("fs", table(std_fs::create(&luau)?))?;
            globals.set("process", table(std_process::create(&luau)?))?;
            globals.set("http", table(std_net_http::create(&luau)?))?;

            globals.set("script", TableBuilder::create(&luau)?
                .with_value("entry_path", "eval")?
                .build()?
            )?;

            if args.len() <= 2 {
                panic!("seal eval got nothing to eval, did you forget to pass in a string?");
            } else {
                args[2].clone()
            }
        } else {
            let file_path = {
                if first_arg == "run" {
                    if args.len() == 2 { // `seal run` (workspace)
                        find_entry_path()
                    } else if args.len() >= 3 { 
                        if args[2].ends_with(".luau") { // `seal run myfile.luau`
                            args[2].clone()
                        } else { // `seal run somearg somearg2` (workspace)
                            find_entry_path()
                        }
                    } else {
                        panic!("seal run: invalid number of arguments provided. Use `seal run` to run the current workspace or `seal run ./somefile.luau` to run a specific file.");
                    }
                } else { // `seal myfile.luau`
                    first_arg.clone()
                }
            };

            if !file_path.ends_with(".luau") {
                panic!("Wrong language! seal only runs .luau files")
            }

            entry_path = file_path.clone();

            globals.set("script", TableBuilder::create(&luau)?
                .with_value("entry_path", file_path.to_owned())?
                .with_function("path", globals::get_script_path)?
                .with_function("parent", globals::get_script_parent)?
                .build()?
            )?;

            let path_metadata = fs::metadata(&file_path);
            match path_metadata {
                Ok(metadata) => {
                    if metadata.is_file() && file_path.ends_with(".luau") {
                        fs::read_to_string(&file_path)?
                    } else if metadata.is_dir() {
                        // we should be able to 'run' directories that contain an init.luau
                        let find_init_filepath = Path::new(&file_path).join("init.luau");
                        if find_init_filepath.exists() {
                            fs::read_to_string(&find_init_filepath)?
                        } else {
                            panic!(r#"seal: Requested file is actually a directory: "{}{}{}"{}{}"#, colors::RESET, &file_path, colors::RED, colors::RESET, "\n  Hint: add a file named 'init.luau' to run this directory itself :)");
                        }
                    } else {
                        panic!(r#"Invalid file extension: expected file path to end with .luau (or be a directory containing an init.luau), got path: "{}{}{}"{}"#, colors::RESET, &file_path, colors::RED, colors::RESET);
                    }
                },
                Err(err) => {
                    panic!("seal: Provided path is Not Ok: {}", err);
                }
            }
        }
    };
    
    // handle shebangs by stripping first line by slicing from first newline
    if luau_code.starts_with("#!") {
        if let Some(first_newline_pos) = luau_code.find('\n') {
            luau_code = luau_code[first_newline_pos + 1..].to_string();
        }
    }

    let script: LuaTable = globals.get("script")?;
    script.set("src", luau_code.to_owned())?;

    globals::set_globals(&luau)?;
    // let cwd = std_env::get_cwd("main.rs")?;
    // require::set_requirer(&luau, cwd, &entry_path)?;

    match luau.load(luau_code).set_name(&entry_path).exec() {
        Ok(()) => {
            std_process::handle_exit_callback(&luau, 0)?;
            Ok(())
        },
        Err(err) => {
            // let replace_main_re = Regex::new(r#"\[string \"[^\"]+\"\]"#).unwrap();
            let mut err_message = error_handling::parse_traceback(err.to_string());
            let script: LuaTable = globals.get("script")?;
            let err_context: Option<String> = script.get("context")?;
            if let Some(context) = err_context {
                let context = format!("{}[CONTEXT] {}{}: {}", colors::BOLD_RED, context, colors::RESET, colors::RED);
                err_message = context + &err_message;
            }
            panic!("{}", err_message);
        },
    }
}

fn seal_setup() -> LuaResult<()> {
    let cwd = std_env::get_cwd("seal setup")?;
    let typedefs_dir = cwd.join(".typedefs");
    let created_typedefs_dir = match fs::create_dir(&typedefs_dir) {
        Ok(_) => true,
        Err(err) => match err.kind() {
            io::ErrorKind::AlreadyExists => {
                println!("seal setup - .typedefs already exists; not replacing it");
                false
            },
            _ => {
                return wrap_err!("seal setup = error creating .typedefs: {}", err);
            }
        }
    };

    if created_typedefs_dir {
        match TYPEDEFS_DIR.extract(typedefs_dir) {
            Ok(()) => {
                println!("seal setup .typedefs in your current directory!");
            },
            Err(err) => {
                return wrap_err!("seal setup - error extracting .typedefs directory: {}", err);
            }
        };
    }

    let seal_setup_settings = include_str!("./scripts/seal_setup_settings.luau");
    let temp_luau = Lua::new();
    globals::set_globals(&temp_luau)?;
    match temp_luau.load(seal_setup_settings).exec() {
        Ok(_) => {
            Ok(())
        },
        Err(err) => {
            wrap_err!("Hit an error running seal_setup_settings.luau: {}", err)
        }
    }
}

fn find_entry_path() -> String {
    let src_dir = Path::new("src");
    let init_luau = src_dir.join("init.luau");
    if init_luau.exists() && init_luau.is_file() {
        init_luau.to_string_lossy().to_string()
    } else if src_dir.exists() && src_dir.is_dir() {
        let main_luau = src_dir.join("main.luau");
        if main_luau.exists() && main_luau.is_file() {
            main_luau.to_string_lossy().to_string()
        } else {
            panic!("seal run: cannot run workspace, missing `@workspace/src/main.luau` or `@workspace/init.luau`\n{}  Tips: use `seal run ./path/to/myfile.luau` or `seal ./path/to/myfile.luau` to run a specific file.\n  To run the current workspace with `seal run`, you must be in the workspace's root path and have a valid entry path (@workspace/src/main.luau or @workspace/init.luau).\n  Run `seal setup` to start a default seal project in your current working directory.", colors::RESET);
        }
    } else {
        panic!("seal run: cannot run workspace, missing `@workspace/src/main.luau` or `@workspace/init.luau`\n{}  Tips: use `seal run ./path/to/myfile.luau` or `seal ./path/to/myfile.luau` to run a specific file.\n  To run the current workspace with `seal run`, you must be in the workspace's root path and have a valid entry path (@workspace/src/main.luau or @workspace/init.luau).\n  Run `seal setup` to start a default seal project in your current working directory.", colors::RESET);
    }
}