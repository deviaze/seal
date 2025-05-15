// pub use crate::{LuaValueResult, LuaEmptyResult, LuaMultiResult, colors, wrap_err};
use mlua::prelude::*;

pub use crate::{std_io::colors as colors, wrap_err, table_helpers::TableBuilder};

pub type LuaValueResult = LuaResult<LuaValue>;
pub type LuaEmptyResult = LuaResult<()>;
pub type LuaMultiResult = LuaResult<LuaMultiValue>;

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
