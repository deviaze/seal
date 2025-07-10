use mluau::prelude::*;
use crate::prelude::*;

fn encode(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "hex.encode(content: string | buffer)";
    let content = match value {
        LuaValue::String(content) => content.as_bytes().to_owned(),
        LuaValue::Buffer(buffy) => buffy.to_vec(),
        other => {
            return wrap_err!("{} expected content to be string or buffer, got: {:?}", function_name, other);
        }
    };
    let encoded = hex::encode(content);
    ok_string(encoded, luau)
}

fn decode(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "hex.decode(encoded: string)";
    let encoded = match value {
        LuaValue::String(s) => s.to_string_lossy(),
        other => {
            return wrap_err!("{} expected encoded to be a string, got: {:?}", function_name, other);
        }
    };
    let decoded = match hex::decode(encoded) {
        Ok(d) => d,
        Err(err) => {
            return wrap_err!("{}: encountered decode err: {}", function_name, err);
        }
    };
    ok_buffy(decoded, luau)
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("encode", encode)?
        .with_function("decode", decode)?
        .build_readonly()
}