use std::env;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use mlua::prelude::*;
use crate::prelude::*;

pub fn get_current_shell() -> String {
    #[cfg(target_family = "unix")]
    {
        // On Unix-like systems, check the SHELL environment variable
        if let Ok(shell_path) = env::var("SHELL") {
            return shell_path;
        }
    }

    #[cfg(target_os = "windows")]
    {
        // first check the SHELL environment variable (if set) if user install custom shell
        if let Ok(shell_path) = env::var("SHELL") {
            return shell_path;
        }
        
        // Check specifically for PowerShell executables
        let pwsh_cmd = "pwsh";
        let powershell_cmd = "powershell";
        
        // check if regular powershell installed bc pwsh 7 blows up
        if let Ok(output) = Command::new("where").arg(powershell_cmd).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return path;
            }
        }
        
        // check if new/oss/powershell 7 installed; it might blow up with threading error tho
        if let Ok(output) = Command::new("where").arg(pwsh_cmd).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return path;
            }
        }
        // fallback to cmd
        if let Ok(shell_path) = env::var("ComSpec") {
            eprintln!("get_current_shell falling to back to cmd.exe; please set $SHELL");
            return shell_path;
        }
    }

    // As a fallback, try to find a shell using `which` or `where` command
    let which_cmd = if cfg!(target_family = "unix") {
        "which"
    } else if cfg!(target_os = "windows") {
        "where"
    } else {
        ""
    };

    if !which_cmd.is_empty() {
        if let Ok(output) = Command::new(which_cmd)
            .arg("sh") // You can replace "sh" with "bash" or "cmd" depending on what you want to check
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return path;
            }
        }
    }

    String::from("")
    // panic!("Could not determine the current shell path");
}

pub fn get_cwd(function_name: &str) -> LuaResult<PathBuf> {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(err) => {
            match err.kind() {
                io::ErrorKind::NotFound => { // yes this happened in testing
                    return wrap_err!("{}: your current directory does not exist (try reloading your terminal/editor?)", function_name);
                },
                io::ErrorKind::PermissionDenied => {
                    return wrap_err!("{}: insufficient permissions to access your current directory", function_name);
                },
                other => {
                    return wrap_err!("{}: error getting your current directory: {}", function_name, other);
                }
            }
        }
    };
    Ok(cwd)
}

fn env_cwd(luau: &Lua, _: LuaValue) -> LuaValueResult {
    let function_name = "env.cwd()";
    let cwd = match get_cwd(function_name) {
        Ok(cwd) => {
            cwd.to_string_lossy().to_string()
        },
        Err(err) => {
            return wrap_err!("{}", err.to_string())
        }
    };
    ok_string(cwd, luau)
}

fn env_environment_getvar(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let var_name = match value {
        LuaValue::String(var) => var.to_string_lossy(),
        other => {
            return wrap_err!("env.getvar expected a string, got: {:#?}", other);
        }
    };

    match env::var(&var_name) {
        Ok(var) => Ok(LuaValue::String(luau.create_string(&var)?)),
        Err(env::VarError::NotPresent) => {
            Ok(LuaNil)
        },
        Err(env::VarError::NotUnicode(_nonunicode_var)) => {
            wrap_err!("env.getvar: requested environment variable '{}' has invalid unicode value", var_name)
        }
    }
}

fn env_environment_setvar(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let key = match multivalue.pop_front() {
        Some(LuaValue::String(key)) => key.to_string_lossy(),
        Some(other) => {
            return wrap_err!("env.setvar(key: string, value: string) expected key to be a string, got: {:#?}", other);
        },
        None => {
            return wrap_err!("env.setvar(key: string, value: string) expected 2 arguments, got none")
        }
    };

    let value = match multivalue.pop_back() {
        Some(LuaValue::String(value)) => value.to_string_lossy(),
        Some(other) => {
            return wrap_err!("env.setvar(key: string, value: string) expected value to be a string, got: {:#?}", other);
        },
        None => {
            return wrap_err!("env.setvar(key: string, value: string) was called with only one argument");
        }
    };

    // safety: setting/removing environment unsafe in multithreaded programs on linux
    // this could be possibly unsafe if the same variable gets set in scripts from multiple thread.spawns on linux
    unsafe { env::set_var(&key, value); }

    match env::var(&key) {
        Ok(_value) => Ok(LuaNil),
        Err(err) => {
            wrap_err!("env.setvar: unable to set environment variable '{}': {}", key, err)
        }
    }

}

fn env_environment_removevar(_luau: &Lua, value: LuaValue) -> LuaValueResult {
    let key = match value {
        LuaValue::String(key) => key.to_string_lossy(),
        other => {
            return wrap_err!("env.removevar(key: string) expected key to be a string, got: {:#?}", other);
        }
    };

    // SAFETY: removing env variable unsafe in multithreaded linux
    // this could cause ub if mixed with thread.spawns 
    unsafe { env::remove_var(&key); }

    match env::var(&key) {
        Ok(key) => {
            wrap_err!("env.removevar: unable to remove environment variable '{}'", key)
        },
        Err(_err) => {
            Ok(LuaNil)
        },
    }
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {

	let formatted_os = match env::consts::OS {
		"linux" => String::from("Linux"),
		"windows" => String::from("Windows"),
		"android" => String::from("Android"),
		"macos" => String::from("MacOS"),
		other => other[0..1].to_uppercase() + &other[1..],
	};

	let mut executable_path = String::from("");
	let mut script_path = String::from("");

	let luau_args = {
		let rust_args: Vec<String> = env::args().collect();
		let result_args = luau.create_table()?;
		for (index, arg) in rust_args.iter().enumerate() {
			if index == 0 {
				executable_path = arg.to_string();
			} else if index == 1 {
				script_path = arg.to_string();
			} else {
				result_args.push(arg.to_string())?;
			}
		}
		result_args
	};

	TableBuilder::create(luau)?
		.with_value("os", formatted_os)?
		.with_value("args", luau_args)?
		.with_value("executable_path", executable_path)?
		.with_value("shell_path", get_current_shell())?
		.with_value("script_path", script_path)?
        .with_function("getvar", env_environment_getvar)?
        .with_function("setvar", env_environment_setvar)?
        .with_function("removevar", env_environment_removevar)?
        .with_function("cwd", env_cwd)?
		.build_readonly()
}