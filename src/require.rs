use crate::*;
use std::fs;

pub fn require(luau: &Lua, path: LuaValue) -> LuaValueResult {
    // convert path to a String
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

// wraps returns of stdlib::create functions with Ok(LuaValue::Table(t))
pub fn ok_table(t: LuaResult<LuaTable>) -> LuaValueResult {
    Ok(LuaValue::Table(t?))
}

pub fn ok_function(f: fn(&Lua, LuaValue) -> LuaValueResult, luau: &Lua) -> LuaValueResult {
    Ok(LuaValue::Function(luau.create_function(f)?))
}

fn get_standard_library(luau: &Lua, path: String) -> LuaValueResult {
    match path.as_str() {
        "@std/fs" => ok_table(std_fs::create(luau)),
        "@std/fs/path" => ok_table(std_fs::pathlib::create(luau)),
        "@std/env" => ok_table(std_env::create(luau)),

        "@std/io" => ok_table(std_io::create(luau)),
        "@std/io/input" => ok_table(std_io_input::create(luau)),
        "@std/io/output" => ok_table(std_io_output::create(luau)),
        "@std/io/colors" => ok_table(colors::create(luau)),
        "@std/io/clear" => ok_function(std_io_output::output_clear, luau),
        "@std/io/format" => ok_function(std_io_output::output_format, luau),
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
        "@std/net/http" => ok_table(std_net_http::create(luau)),
        "@std/net/http/server" => ok_table(std_net_serve::create(luau)),
        "@std/net/request" => ok_function(std_net_http::http_request, luau),

        "@std/crypt" => ok_table(std_crypt::create(luau)),
        "@std/crypt/aes" => ok_table(std_crypt::create_aes(luau)),
        "@std/crypt/rsa" => ok_table(std_crypt::create_rsa(luau)),
        "@std/crypt/hash" => ok_table(std_crypt::create_hash(luau)),
        "@std/crypt/password" => ok_table(std_crypt::create_password(luau)),

        "@std/thread" => ok_table(std_thread::create(luau)),

        "@std/testing" => ok_table(std_testing::create(luau)),
        "@std/testing/try" => ok_function(std_testing::testing_try, luau),

        "@std" => {
            ok_table(TableBuilder::create(luau)?
                .with_value("fs", std_fs::create(luau)?)?
                .with_value("env", std_env::create(luau)?)?
                .with_value("io", std_io::create(luau)?)?
                .with_value("colors", colors::create(luau)?)?
                .with_function("format", std_io_output::format_output)?
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

fn resolve_path(luau: &Lua, path: String) -> LuaResult<String> {
    let require_resolver = include_str!("./scripts/require_resolver.luau");
    let r: LuaFunction = luau.load(require_resolver).eval()?;
    match r.call::<LuaValue>(path.to_owned()) {
        // Ok(LuaValue::String(path)) => Ok(path.to_string_lossy()),
        Ok(LuaValue::Table(result_table)) => {
            let ok: bool = result_table.raw_get("ok")?;
            if ok {
                let resolved_path: LuaString = result_table.raw_get("resolved_path")?;
                Ok(resolved_path.to_string_lossy())
            } else {
                let err_message: LuaString = result_table.raw_get("err")?;
                let err_message = err_message.to_string_lossy();
                wrap_err!("require: {}", err_message)
            }
        },
        Ok(_other) => {
            panic!("require: ./scripts/require_resolver.luau returned something that isn't a string or err table; this shouldn't be possible");
        },
        Err(err) => {
            panic!("require: ./scripts/require_resolver.luau broke? this shouldn't happen; err: {}", err);
        }
    }
}
