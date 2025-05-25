use mlua::prelude::*;
use crate::prelude::*;
use std::fs;
use crate::std_json;
use crate::std_fs::{validate_path, entry::{wrap_io_read_errors, wrap_io_read_errors_empty}};
use serde_yml::Value as YamlValue;

fn encode(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "yaml.encode(content: { [any]: any })";
    let table_to_encode = match value {
        LuaValue::Table(t) => t,
        other => {
            return wrap_err!("{} expected content to be a table, got: {:?}", function_name, other);
        }
    };
    let json_encode_options = std_json::EncodeOptions::default();
    let intermediate_json_representation = match std_json::encode(luau, table_to_encode, json_encode_options) {
        Ok(json) => json,
        Err(err) => {
            return wrap_err!("{}: error serializing table to intermediate json representation (needed to convert to yaml): {}", function_name, err);
        }
    };
    let yaml_value = match serde_yml::from_str::<serde_yml::Value>(&intermediate_json_representation) {
        Ok(yaml) => yaml,
        Err(err) => {
            return wrap_err!("{}: unable to serialize to YamlValue due to err: {}", function_name, err);
        }
    };
    let encoded = match serde_yml::to_string(&yaml_value) {
        Ok(encoded) => encoded,
        Err(err) => {
            return wrap_err!("{}: unable to convert YamlValue back to a string due to err: {}", function_name, err);
        }
    };
    ok_string(encoded, luau)
}

fn decode(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "yaml.decode(encoded: string)";
    let encoded = match value {
        LuaValue::String(s) => s.to_str()?.to_string(),
        other => {
            return wrap_err!("{} expected encoded to be a string, got: {:?}", function_name, other);
        }
    };
    let yaml_value = match serde_yml::from_str::<serde_yml::Value>(&encoded) {
        Ok(encoded) => encoded,
        Err(err) => {
            return wrap_err!("{}: error serializing content into YamlValue with serde_yml: {}", function_name, err);
        }
    };
    let intermediate_json_representation = match serde_json_lenient::to_string(&yaml_value) {
        Ok(s) => s,
        Err(err) => {
            return wrap_err!("{}: error converting YamlValue into intermediate json string representation: {}", function_name, err);
        }
    };
    std_json::json_decode(luau, intermediate_json_representation)
}

fn yaml_readfile(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "yaml.readfile(path: string)";
    let path = match value {
        LuaValue::String(path) => validate_path(&path, function_name)?,
        other => {
            return wrap_err!("{} expected path to be string, got: {:?}", function_name, other);
        }
    };
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) => {
            return wrap_io_read_errors(err, function_name, path);
        }
    };
    let yaml_value: YamlValue = match serde_yml::from_str(&content) {
        Ok(yaml) => yaml,
        Err(err) => {
            return wrap_err!("{}: unable to decode due to err: {}", function_name, err);
        }
    };
    let intermediate_json_representation = match serde_json_lenient::to_string(&yaml_value) {
        Ok(s) => s,
        Err(err) => {
            return wrap_err!("{}: error converting YamlValue into intermediate json string representation: {}", function_name, err);
        }
    };
    std_json::json_decode(luau, intermediate_json_representation)
}

fn yaml_writefile(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let function_name = "yaml.writefile(path: string, content: { [any]: any })";
    let path = match multivalue.pop_front() {
        Some(LuaValue::String(path)) => validate_path(&path, function_name)?,
        Some(other) => {
            return wrap_err!("{} expected path to be a string, got: {:?}", function_name, other);
        }
        None => {
            return wrap_err!("{} called without required argument 'path'", function_name);
        }
    };
    let table_to_encode = match multivalue.pop_front() {
        Some(LuaValue::Table(t)) => t,
        Some(other) => {
            return wrap_err!("{} expected content to be a table, got: {:?}", function_name, other);
        }
        None => {
            return wrap_err!("{} called without required argument 'content'", function_name);
        }
    };
    let json_encode_options = std_json::EncodeOptions::default();
    let intermediate_json_representation = match std_json::encode(luau, table_to_encode, json_encode_options) {
        Ok(json) => json,
        Err(err) => {
            return wrap_err!("{}: error serializing table to intermediate json representation (needed to convert to yaml): {}", function_name, err);
        }
    };
    let yaml_value = match serde_yml::from_str::<serde_yml::Value>(&intermediate_json_representation) {
        Ok(yaml) => yaml,
        Err(err) => {
            return wrap_err!("{}: unable to serialize to YamlValue due to err: {}", function_name, err);
        }
    };
    let encoded = match serde_yml::to_string(&yaml_value) {
        Ok(encoded) => encoded,
        Err(err) => {
            return wrap_err!("{}: unable to convert YamlValue back to a string due to err: {}", function_name, err);
        }
    };
    match fs::write(&path, encoded) {
        Ok(_) => Ok(()),
        Err(err) => wrap_io_read_errors_empty(err, function_name, path)
    }
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("encode", encode)?
        .with_function("decode", decode)?
        .with_function("readfile", yaml_readfile)?
        .with_function("writefile", yaml_writefile)?
        .build_readonly()
}