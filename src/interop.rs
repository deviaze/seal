use mlua::prelude::*;
use crate::{table_helpers::TableBuilder, LuaValueResult, colors, wrap_err};

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

fn interop_require_load(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let chunk_name = match multivalue.pop_front() {
        Some(LuaValue::String(s)) => {
            s.to_string_lossy()
        },
        Some(other) => {
            return wrap_err!("expected chunk_name to be a string, got {:?}", other);
        },
        None => {
            return wrap_err!("expected chunk_name, got nothing");
        }
    };
    let to_path = match multivalue.pop_front() {
        Some(LuaValue::String(s)) => {
            s.to_string_lossy()
        },
        Some(other) => {
            return wrap_err!("expected to_path to be a string, got {:?}", other);
        },
        None => {
            return wrap_err!("expected to_path, got nothing");
        }
    };

    crate::require::load(luau, &chunk_name, &to_path)
}

fn interop_get_luaurc(luau: &Lua, chunk_name: String) -> LuaValueResult {
    crate::require::get_luaurc(luau, &chunk_name)
}

pub fn create_mlua(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("isint", interop_mlua_isint)?
        .with_function("iserror", interop_mlua_iserror)?
        .build_readonly()
}

pub fn create_require(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("load", interop_require_load)?
        .with_function("get_luaurc", interop_get_luaurc)?
        .build_readonly()
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_value("mlua", create_mlua(luau)?)?
        .with_value("require", create_require(luau)?)?
        .with_function("navigate_path", interop_require_load)?
        .with_function("get_luaurc", interop_get_luaurc)?
        .build_readonly()
}