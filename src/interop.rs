use std::path::PathBuf;

use mluau::prelude::*;
use crate::prelude::*;
use crate::compile;
use crate::std_fs::validate_path;

fn interop_mlua_isint(_luau: &Lua, n: LuaValue) -> LuaValueResult {
    match n {
        LuaValue::Integer(_i) => {
            Ok(LuaValue::Boolean(true))
        },
        LuaValue::Number(_n) => {
            Ok(LuaValue::Boolean(false))
        },
        other => {
            wrap_err!("interop.mlua.isint(n: number) expected n to be a number, got: {:#?}", other)
        }
    }
}

fn interop_mlua_iserror(_luau: &Lua, value: LuaValue) -> LuaValueResult {
    match value {
        LuaValue::Error(_err) => {
            Ok(LuaValue::Boolean(true))
        },
        _other => {
            Ok(LuaValue::Boolean(false))
        }
    }
}

fn interop_standalone_check(_luau: &Lua, value: LuaValue) -> LuaResult<bool> {
    let function_name = "standalone.check(path: string)";
    let path = match value {
        LuaValue::String(path) => {
            PathBuf::from(validate_path(&path, function_name)?)
        },
        other => {
            return wrap_err!("{} expected path to be a string, got: {:?}", function_name, other);
        }
    };
    Ok(compile::is_standalone(Some(path)))
}

fn interop_standalone_extract(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "standalone.extract(path: string)";
    let path = match value {
        LuaValue::String(path) => {
            PathBuf::from(validate_path(&path, function_name)?)
        },
        other => {
            return wrap_err!("{} expected path to be a string, got: {:?}", function_name, other);
        }
    };
    if let Some(bytecode) = compile::extract_bytecode(Some(path)) {
        ok_buffy(&bytecode, luau)
    } else {
        wrap_err!("{}: bytecode could not be extracted :/ check your path?", function_name)
    }
}

fn interop_standalone_eval(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "standalone.eval(path: string, chunk_name: string)";
    let path = match multivalue.pop_front() {
        Some(LuaValue::String(path)) => {
            PathBuf::from(validate_path(&path, function_name)?)
        },
        Some(LuaNil) | None => {
            return wrap_err!("{} incorrectly called with zero arguments", function_name);
        }
        Some(other) => {
            return wrap_err!("{} expected path to be a string, got: {:?}", function_name, other);
        }
    };
    let chunk_name = match multivalue.pop_front() {
        Some(LuaValue::String(chunk_name)) => {
            chunk_name.to_string_lossy()
        },
        Some(LuaNil) | None => {
            return wrap_err!("{} called without required argument 'chunk_name'", function_name);
        }
        Some(other) => {
            return wrap_err!("{} expected chunk_name to be a string, got: {:?}", function_name, other);
        }
    };

    let Some(bytecode) = compile::extract_bytecode(Some(path)) else {
        return wrap_err!("{}: unable to extract bytecode", function_name);
    };

    match luau.load(&bytecode).set_name(&chunk_name).eval::<LuaValue>() {
        Ok(value) => Ok(value),
        Err(err) => {
            wrap_err!("{}: error evaluating bytecode: {}", function_name, err)
        }
    }
}

pub fn create_standalone(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("check", interop_standalone_check)?
        .with_function("extract", interop_standalone_extract)?
        .with_function("eval", interop_standalone_eval)?
        .build_readonly()
}

pub fn create_mlua(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("isint", interop_mlua_isint)?
        .with_function("iserror", interop_mlua_iserror)?
        .build_readonly()
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_value("mlua", create_mlua(luau)?)?
        .with_value("standalone", create_standalone(luau)?)?
        .build_readonly()
}