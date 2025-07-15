use mluau::prelude::*;
use crate::prelude::*;

// base64 lib is kinda annoying so
use base64::{engine::general_purpose::{STANDARD as base64_standard, URL_SAFE as base64_url_safe}, Engine};

/// encodes luau string | buffer to base64 STANDARD
fn encode(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "base64.encode(content: string | buffer)";
    let value_to_serialize = match value {
        LuaValue::String(s) => s.as_bytes().to_owned(),
        LuaValue::Buffer(buffy) => buffy.to_vec(),
        other => {
            return wrap_err!("{} expected content to be a string or buffer, got: {:?}", function_name, other);
        }
    };
    let encoded = base64_standard.encode(value_to_serialize);
    ok_string(encoded, luau)
}

fn decode(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "base64.decode(encoded: string)";
    let encoded = match value {
        LuaValue::String(s) => s.to_string_lossy(),
        other => {
            return wrap_err!("{} expected encoded to be a string, got: {:?}", function_name, other);
        }
    };
    let decoded = match base64_standard.decode(encoded) {
        Ok(dec) => dec,
        Err(err) => {
            return wrap_err!("{}: error decoding base64 using STANDARD Engine: {}", function_name, err);
        }
    };
    ok_buffy(decoded, luau)
}

fn urlsafe_encode(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "base64.urlsafe.encode(content: string | buffer)";
    let content = match value {
        LuaValue::String(s) => s.as_bytes().to_owned(),
        LuaValue::Buffer(buffy) => buffy.to_vec(),
        other => {
            return wrap_err!("{} expected content to be string or buffer, got: {:?}", function_name, other);
        }
    };
    let encoded = base64_url_safe.encode(content);
    ok_string(encoded, luau)
}

fn urlsafe_decode(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "base64.urlsafe.decode(encoded: string)";
    let encoded = match value {
        LuaValue::String(s) => s.to_string_lossy(),
        other => {
            return wrap_err!("{} expected encoded to be a string, got: {:?}", function_name, other);
        }
    };
    let decoded = match base64_url_safe.decode(encoded) {
        Ok(dec) => dec,
        Err(err) => {
            return wrap_err!("{}: error decoding base64 using URL_SAFE Engine: {}", function_name, err);
        }
    };
    ok_buffy(decoded, luau)
}

fn create_urlsafe(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("encode", urlsafe_encode)?
        .with_function("decode", urlsafe_decode)?
        .build_readonly()
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("encode", encode)?
        .with_function("decode", decode)?
        .with_value("urlsafe", create_urlsafe(luau)?)?
        .build_readonly()
}