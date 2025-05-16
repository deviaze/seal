use mlua::prelude::*;
use crate::table_helpers::TableBuilder;

pub mod colors;
pub mod input;
pub mod output;

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_value("input", self::input::create(luau)?)?
        .with_value("colors", self::colors::create(luau)?)?
        .with_value("output", self::output::create(luau)?)?
        .build_readonly()
}