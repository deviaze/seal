use mluau::prelude::*;
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

pub fn create_mlua(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("isint", interop_mlua_isint)?
        .with_function("iserror", interop_mlua_iserror)?
        .build_readonly()
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_value("mlua", create_mlua(luau)?)?
        .build_readonly()
}