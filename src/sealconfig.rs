use std::path::PathBuf;
use std::{fs, io};

use crate::prelude::*;
use crate::std_env;
use crate::std_fs::validate_path;
use mluau::prelude::*;

pub struct SealConfig {
    pub entry_path: String,
    pub test_path: Option<String>,
}

impl SealConfig {
    pub fn read(luau: &Lua, current_path: Option<PathBuf>, function_name: &'static str) -> LuaResult<Option<Self>> {
        let mut current_path = current_path.unwrap_or(std_env::get_cwd(function_name)?);

        while current_path.exists() {
            let seal_dir = current_path.join(".seal");
            let config_path = seal_dir.join("config.luau");

            if seal_dir.is_dir() && config_path.is_file() {
                current_path = config_path;
                break;
            } else if seal_dir.is_dir() && !config_path.exists() {
                if std_env::get_cwd(function_name)?
                    .join("src")
                    .join("main.luau")
                    .exists() 
                {
                    return Ok(Some(SealConfig { entry_path: String::from("./src/main.luau"), test_path: None }))
                } else {
                    return Ok(None);
                }
            } else if let Some(parent) = current_path.parent() {
                current_path = parent.to_path_buf();
            } else {
                return Ok(None);
            }
        }

        let sealconfig_src = match fs::read(&current_path) {
            Ok(contents) => contents,
            Err(err) => {
                // i just inlined wrap_io_read_errors lol
                return match err.kind() {
                    io::ErrorKind::NotFound =>
                        wrap_err!("{}: File/directory not found: '{}'", function_name, current_path.display()),
                    io::ErrorKind::PermissionDenied =>
                        wrap_err!("{}: Permission denied: '{}'", function_name, current_path.display()),
                    _other => {
                        wrap_err!("{}: Error on path: '{}': {}", function_name, current_path.display(), err)
                    }
                };
            },
        };

        let sealconfig = match luau.load(sealconfig_src).eval::<LuaValue>() {
            Ok(LuaValue::Table(config)) => config,
            Ok(other) => {
                return wrap_err!("{}: config.luau at '{}' returned something that isn't a table: {:?}", function_name, current_path.display(), other);
            },
            Err(err) => {
                return wrap_err!("{}: unable to load config.luau at '{}' due to err: {}", function_name, current_path.display(), err);
            }
        };

        let entry_path = match sealconfig.raw_get("entry_path")? {
            LuaValue::String(p) => validate_path(&p, function_name)?,
            LuaNil => String::from("./src/main.luau"),
            other => {
                return wrap_err!("{}: unexpected result when reading config.luau at '{}', \
                field entry_path expected to be a string, got: {:?}", function_name, current_path.display(), other);
            }
        };

        let test_path = match sealconfig.raw_get("test_path")? {
            LuaValue::String(test_path) => Some(validate_path(&test_path, function_name)?),
            LuaNil => None,
            other => {
                return wrap_err!("{}: unexpected test_path when reading config.luau at '{}'; \
                test_path expected to be a string, got: {:?}", function_name, current_path.display(), other);
            }
        };

        Ok(Some(SealConfig { entry_path, test_path }))
    }
}
