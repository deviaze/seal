use requirer::FsRequirer;

use mlua::prelude::*;
use crate::{*, std_json::json_decode};
use std::{collections::VecDeque, fs, path::PathBuf, io};

mod requirer;

pub fn require(luau: &Lua, path: LuaValue) -> LuaValueResult {
    let path = match path {
        LuaValue::String(path) => path.to_string_lossy(),
        other => {
            return wrap_err!("require expected a string path (like \"@std/json\" or \"./relative_file\"), got: {:#?}", other);
        }
    };

    if path.starts_with("@std") || path.starts_with("@interop") {
        get_standard_library(luau, path)
    } else {
        let path = resolve_path(luau, path)?;
        let require_cache: LuaTable = luau.globals().raw_get("_REQUIRE_CACHE").unwrap();
        let cached_result: Option<LuaValue> = require_cache.raw_get(path.clone())?;

        if let Some(cached_result) = cached_result {
            Ok(cached_result)
        } else {
            let data = match fs::read_to_string(&path) {
                Ok(data) => data,
                Err(err) => {
                    match err.kind() {
                        io::ErrorKind::NotFound => {
                            return wrap_err!("require: no such file or directory for resolved path {}", path);
                        },
                        _other => {
                            return wrap_err!("require: error reading file: {}", err);
                        }
                    }
                }
            };
            let result: LuaValue = luau.load(data).set_name(&path).eval()?;
            require_cache.raw_set(path.clone(), result)?;
            // this is pretty cursed but let's just read the data we just wrote to the cache to get a new LuaValue
            // that can be returned without breaking the borrow checker or cloning
            let result = require_cache.raw_get(path.to_owned())?;
            Ok(result)
        }
    }
}

fn get_standard_library(luau: &Lua, path: String) -> LuaValueResult {
    match path.as_str() {
        "@std/fs" => ok_table(std_fs::create(luau)),
        "@std/fs/path" => ok_table(std_fs::pathlib::create(luau)),
        "@std/fs/file" => ok_table(std_fs::filelib::create(luau)),
        "@std/fs/dir" => ok_table(std_fs::dirlib::create(luau)),

        "@std/env" => ok_table(std_env::create(luau)),

        "@std/io" => ok_table(std_io::create(luau)),
        "@std/io/input" => ok_table(std_io::input::create(luau)),
        "@std/io/output" => ok_table(std_io::output::create(luau)),
        "@std/io/colors" => ok_table(colors::create(luau)),
        "@std/io/clear" => ok_function(std_io::output::clear, luau),
        "@std/io/format" => ok_function(std_io::output::format, luau),
        "@std/colors" => ok_table(colors::create(luau)),

        "@std/time" => ok_table(std_time::create(luau)),
        "@std/time/datetime" => ok_table(std_time::create_datetime(luau)),
        "@std/datetime" => ok_table(std_time::create_datetime(luau)),

        "@std/process" => ok_table(std_process::create(luau)),

        "@std/serde" => ok_table(std_serde::create(luau)),
        "@std/serde/base64" => ok_table(std_serde::create_base64(luau)),
        "@std/serde/toml" => ok_table(std_serde::create_toml(luau)),
        "@std/serde/yaml" => ok_table(std_serde::create_yaml(luau)),
        "@std/serde/json" => ok_table(std_json::create(luau)),
        "@std/serde/hex" => ok_table(std_serde::create_hex(luau)),
        "@std/json" => ok_table(std_json::create(luau)),

        "@std/net" => ok_table(std_net::create(luau)),
        "@std/net/http" => ok_table(std_net::http::create(luau)),
        "@std/net/http/server" => ok_table(std_net::serve::create(luau)),
        "@std/net/request" => ok_function(std_net::http::request, luau),

        "@std/crypt" => ok_table(std_crypt::create(luau)),
        "@std/crypt/aes" => ok_table(std_crypt::create_aes(luau)),
        "@std/crypt/rsa" => ok_table(std_crypt::create_rsa(luau)),
        "@std/crypt/hash" => ok_table(std_crypt::create_hash(luau)),
        "@std/crypt/password" => ok_table(std_crypt::create_password(luau)),

        "@std/str_internal" => ok_table(std_str_internal::create(luau)),
        "@std/str" => ok_table(load_std_str(luau)),

        "@std/thread" => ok_table(std_thread::create(luau)),

        "@std/testing" => ok_table(std_testing::create(luau)),
        "@std/testing/try" => ok_function(std_testing::testing_try, luau),

        "@std" => {
            ok_table(TableBuilder::create(luau)?
                .with_value("fs", std_fs::create(luau)?)?
                .with_value("str", load_std_str(luau)?)?
                .with_value("env", std_env::create(luau)?)?
                .with_value("io", std_io::create(luau)?)?
                .with_value("colors", colors::create(luau)?)?
                .with_function("format", std_io::output::format)?
                .with_value("time", std_time::create(luau)?)?
                .with_value("datetime", std_time::create_datetime(luau)?)?
                .with_value("process", std_process::create(luau)?)?
                .with_value("serde", std_serde::create(luau)?)?
                .with_value("json", std_json::create(luau)?)?
                .with_value("net", std_net::create(luau)?)?
                .with_value("crypt", std_crypt::create(luau)?)?
                .with_value("thread", std_thread::create(luau)?)?
                .with_value("testing", std_testing::create(luau)?)?
                .build_readonly()
            )
        },
        "@interop" => ok_table(interop::create(luau)),
        "@interop/mlua" => ok_table(interop::create_mlua(luau)),
        other => {
            wrap_err!("program required an unexpected standard library: {}", other)
        }
    }
}

const STD_STR_SRC: &str = include_str!("../std_str.luau");
fn load_std_str(luau: &Lua) -> LuaResult<LuaTable> {
    luau.load(STD_STR_SRC).eval::<LuaTable>()
}

fn resolve_path(luau: &Lua, path: String) -> LuaResult<String> {
    let resolver_src = include_str!("./resolver.luau");
    let LuaValue::Table(resolver) = luau.load(resolver_src).eval()? else {
        panic!("require resolver didnt return table??");
    };
    let LuaValue::Function(resolve) = resolver.raw_get("resolve")? else {
        panic!("require resolver.resolve not a function??");
    };
    match resolve.call::<LuaValue>(path.to_owned()) {
        Ok(LuaValue::Table(result_table)) => {
            if let LuaValue::String(path) = result_table.raw_get("path")? {
                Ok(path.to_string_lossy())
            } else if let LuaValue::String(err) = result_table.raw_get("err")? {
                wrap_err!("require: {}", err.to_string_lossy())
            } else {
                panic!("require: resolve() returned an unexpected table {:#?}", result_table);
            }
        },
        Ok(_other) => {
            panic!("require: resolve() returned something that isn't a string or err table; this shouldn't be possible");
        },
        Err(err) => {
            panic!("require: resolve() broke? this shouldn't happen; err: {}", err);
        }
    }
}

fn get_require_cache(luau: &Lua) -> LuaResult<LuaTable> {
    // luau.globals().raw_get::<LuaTable>("_REQUIRE_CACHE")
    let require_cache = match luau.globals().raw_get("_REQUIRE_CACHE")? {
        LuaValue::Table(t) => t,
        other => {
            return wrap_err!("expected globals._REQUIRE_CACHE, got: {:?}", other);
        }
    };
    Ok(require_cache)
}

pub fn _set_requirer(luau: &Lua, cwd: PathBuf, entrypoint_chunk: &str) -> LuaEmptyResult {
    let current_chunk = PathBuf::from(entrypoint_chunk);
    let requirer = FsRequirer::from(cwd, current_chunk, get_require_cache(luau)?);
    let require_function = match luau.create_require_function(requirer) {
        Ok(f) => f,
        Err(err) => {
            return wrap_err!("unable to create requirer function because {}", err);
        }
    };
    luau.globals().raw_set("_REQUIRE_FUNCTION", require_function)?;
    Ok(())
}

fn extract_alias<'a>(to_path: &'a str, function_name: &str) -> LuaResult<&'a str> {
    let requested_path = to_path;
    let to_path = to_path.trim_start_matches('@');
    if to_path.contains("/") {
        let mut components: VecDeque<&str> = to_path.split('/').collect();
        if let Some(alias) = components.pop_front() {
            Ok(alias)
        } else {
            wrap_err!("{}: unable to extract alias from requested path '{}'", requested_path, function_name)
        }
    } else {
        Ok(to_path)
    }
}

pub fn load(luau: &Lua, chunk_name: &str, to_path: &str) -> LuaValueResult {
    // let requested_path = to_path;
    let chunk_name = PathBuf::from(chunk_name);
    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(err) => {
            return wrap_err!("navigate path cant get current dir: {}", err);
        }
    };
    let temp_requirer = FsRequirer::from(cwd, chunk_name, get_require_cache(luau)?);
    if to_path.starts_with("@") {
        let aliases = temp_requirer.get_aliases()?;
        let alias = extract_alias(to_path, "require.load")?;
        let to_path = to_path.trim_start_matches('@').replace(alias, "");
        if let Some(replacement_path) = aliases.get(alias) {
            let replacement_path = replacement_path.trim_end_matches('/');
            let to_path = replacement_path.to_owned() + &to_path;
            temp_requirer.to_luaurc();
            match temp_requirer.to_child(&to_path) {
                Ok(_) => {
                    temp_requirer.load(luau)
                },
                Err(_) => {
                    let shown_path = temp_requirer.show_current_path();
                    wrap_err!("Can't navigate from '{}' to '{}' due to navigate err", shown_path, to_path)
                }
            }
        } else {
            wrap_err!("Can't find extracted alias '{}' in .luaurc", alias)
        }
    } else {
        println!("{:#?}", temp_requirer);
        let mut to_path = to_path.to_owned();
        if to_path.starts_with("..") {
            let _ = temp_requirer.to_parent();
            to_path = to_path.replace("..", ".");
        }
        println!("{}", to_path);
        match temp_requirer.to_child(&to_path) {
            Ok(_) => {
                temp_requirer.load(luau)
            },
            Err(_) => {
                let shown_path = temp_requirer.show_current_path();
                wrap_err!("Can't navigate from '{}' to '{}' due to navigate err", shown_path, to_path)
            }
        }
    }
}

pub fn get_luaurc(luau: &Lua, chunk_name: &str) -> LuaValueResult {
    let current_chunk = PathBuf::from(chunk_name);
    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(err) => {
            return wrap_err!("navigate path cant get current dir: {}", err);
        }
    };
    let temp_requirer = FsRequirer::from(cwd, current_chunk, get_require_cache(luau)?);
    let luaurc_contents = temp_requirer.read_luaurc()?;
    json_decode(luau, luaurc_contents)
}