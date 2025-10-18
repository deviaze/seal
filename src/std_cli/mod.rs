use mluau::prelude::*;
use crate::prelude::*;

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        // .with_value("prompt", prompt::create(luau)?)?
        .build_readonly()
}