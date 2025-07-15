use mluau::prelude::*;

pub mod http;
pub mod serve;

use crate::prelude::*;

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_value("http", self::http::create(luau)?)?
        .build_readonly()
}
