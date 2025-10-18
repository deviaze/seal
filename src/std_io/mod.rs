
use mluau::prelude::*;
use crate::prelude::*;

pub mod colors;
pub mod input;
pub mod output;
pub mod format;
pub mod prompt;

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_value("input", self::input::create(luau)?)?
        .with_value("colors", self::colors::create(luau)?)?
        .with_value("output", self::output::create(luau)?)?
        .with_value("prompt", self::prompt::create(luau)?)?
        .build_readonly()
}