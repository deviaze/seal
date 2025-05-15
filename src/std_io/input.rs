#![allow(dead_code)]

use mlua::prelude::*;
use crate::table_helpers::TableBuilder;
use std::io::{self, Write};

type LuaValueResult = LuaResult<LuaValue>;

fn input_get(luau: &Lua, raw_prompt: Option<String>) -> LuaValueResult {
    if let Some(prompt) = raw_prompt {
        print!("{}", prompt);
        io::stdout().flush().unwrap();
    }

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim_end().to_string();

    Ok(LuaValue::String(luau.create_string(&input)?))
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("get", input_get)?
        .build_readonly()
}
