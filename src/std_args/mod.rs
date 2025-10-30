use mluau::prelude::*;
use crate::prelude::*;

use crate::std_err::ecall;

const ARGS_DOT_LUAU: &str = include_str!("./args.luau");

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    let t = TableBuilder::create(luau)?
        // .with_value("raw", std::env::args_os().collect::<Vec<u8>>())?
        .build()?;

    let prompt_table = match luau.load(ARGS_DOT_LUAU).eval::<LuaTable>() {
        Ok(t) => t,
        Err(err) => {
            panic!("std/args' args.luau did a bad: {}", err);
        }
    };

    for pair in prompt_table.pairs() {
        let (key, value): (String, LuaFunction) = pair?;
        t.raw_set(key, ecall(luau, value)?)?;
    }

    t.set_readonly(true);

    Ok(t)
}