use mlua::prelude::*;
use crate::{prelude::*, std_fs::{entry::{wrap_io_read_errors, wrap_io_read_errors_empty}, validate_path}};
use toml::Value as TomlValue;
use std::fs;

fn encode(_luau: &Lua, value: LuaValue) -> LuaResult<String> {
    let function_name = "toml.encode(t: { [any]: any })";
    let table_to_encode = match value {
        LuaValue::Table(t) => t,
        other => {
            return wrap_err!("{} expected t to be a table, got: {:?}", function_name, other);
        }
    };
    
    let toml_value = luau_to_toml(LuaValue::Table(table_to_encode), function_name)?;
    let encoded = match toml::to_string::<TomlValue>(&toml_value) {
        Ok(e) => e,
        Err(err) => {
            return wrap_err!("{}: unable to serialize to toml: {}", function_name, err);
        }
    };

    Ok(encoded)
}

fn decode(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "toml.decode(encoded: string)";
    let encoded = match value {
        LuaValue::String(s) => s.to_string_lossy(),
        other => {
            return wrap_err!("{} expected encoded to be a string, got: {:?}", function_name, other);
        }
    };
    let toml_value: TomlValue = match toml::from_str(&encoded) {
        Ok(toml) => toml,
        Err(err) => {
            return wrap_err!("{}: unable to decode due to err: {}", function_name, err);
        }
    };
    toml_to_luau(luau, toml_value)
}

fn toml_readfile(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "toml.readfile(path: string)";
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
    let toml_value: TomlValue = match toml::from_str(&content) {
        Ok(toml) => toml,
        Err(err) => {
            return wrap_err!("{}: unable to decode due to err: {}", function_name, err);
        }
    };
    toml_to_luau(luau, toml_value)
}

fn toml_writefile(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let function_name = "toml.writefile(path: string, content: { [any]: any })";
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
    let toml_value = luau_to_toml(LuaValue::Table(table_to_encode), function_name)?;
    let encoded = match toml::to_string::<TomlValue>(&toml_value) {
        Ok(encoded) => encoded,
        Err(err) => {
            return wrap_err!("{}: unable to serialize toml due to err: {}", function_name, err);
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
        .with_function("writefile", toml_writefile)?
        .with_function("readfile", toml_readfile)?
        .build_readonly()
}

fn toml_to_luau(luau: &Lua, value: TomlValue) -> LuaValueResult {
    match value {
        TomlValue::String(s) => Ok(LuaValue::String(luau.create_string(&s)?)),
        TomlValue::Integer(i) => Ok(LuaValue::Integer(i)),
        TomlValue::Float(f) => Ok(LuaValue::Number(f)),
        TomlValue::Boolean(b) => Ok(LuaValue::Boolean(b)),
        TomlValue::Datetime(dt) => Ok(LuaValue::String(luau.create_string(dt.to_string())?)),
        TomlValue::Array(arr) => {
            let luau_table = luau.create_table()?;
            for (i, v) in arr.into_iter().enumerate() {
                luau_table.set(i + 1, toml_to_luau(luau, v)?)?;
            }
            ok_table(Ok(luau_table))
        }
        TomlValue::Table(table) => {
            let luau_table = luau.create_table()?;
            for (k, v) in table.into_iter() {
                luau_table.set(k, toml_to_luau(luau, v)?)?;
            }
            ok_table(Ok(luau_table))
        }
    }
}

fn luau_to_toml(value: LuaValue, function_name: &'static str) -> LuaResult<TomlValue> {
    let toml_value = match value {
        LuaValue::String(s) => TomlValue::String(s.to_string_lossy()),
        LuaValue::Integer(i) => TomlValue::Integer(i),
        LuaValue::Number(n) => TomlValue::Float(n),
        LuaValue::Boolean(b) => TomlValue::Boolean(b),
        LuaValue::Table(t) => {
            if t.raw_len() > 0 {
                convert_luau_array_to_toml(t, function_name)?
            } else {
                convert_luau_map_to_toml(t, function_name)?
            }
        },
        other => {
            return wrap_err!("{}: unsupported luau type: {:?}", function_name, other);
        }
    };
    Ok(toml_value)
}

fn convert_luau_array_to_toml(t: LuaTable, function_name: &'static str) -> LuaResult<TomlValue> {
    let mut toml_array = Vec::new();
    for i in 1..=t.raw_len() {
        let item = t.get(i)?;
        toml_array.push(luau_to_toml(item, function_name)?);
    }
    Ok(TomlValue::Array(toml_array))
}

fn convert_luau_map_to_toml(t: LuaTable, function_name: &'static str) -> LuaResult<TomlValue> {
    let mut toml_map = toml::map::Map::new();
    for pair in t.pairs::<LuaValue, LuaValue>() {
        let (key, value) = pair?;
        let key_str = match key {
            LuaValue::String(s) => s.to_str()?.to_string(),
            other => return wrap_err!("{}: error serializing to toml map: key must be a string, got: {:?}", function_name, other),
        };
        let toml_value = luau_to_toml(value, function_name)?;
        toml_map.insert(key_str, toml_value);
    }
    Ok(toml::Value::Table(toml_map))
}
