use crate::{colors, std_fs, std_json, table_helpers::TableBuilder, LuaEmptyResult, LuaValueResult};
use mlua::prelude::*;
use toml::Value as TomlValue;
use serde_yaml::Value as YamlValue;
use serde_json_lenient as serde_json;

fn serde_yaml_decode(luau: &Lua, value: LuaValue) -> LuaValueResult {
    match value {
        LuaValue::String(data) => {
            let yaml_value: YamlValue = match serde_yaml::from_str(&data.to_string_lossy()) {
                Ok(yaml) => yaml,
                Err(err) => {
                    return wrap_err!("serde.yaml.decode: Error converting to yaml: {}", err);
                }
            };
            let json_string = match serde_json::to_string(&yaml_value) {
                Ok(json) => json,
                Err(err) => {
                    return wrap_err!("serde.yaml.decode: Error converting yaml to json to convert to luau table: {}", err);
                }
            };
            Ok(std_json::json_decode(luau, json_string)?)
        }
        other => {
            wrap_err!("serde.yaml.decode expected string, got: {:#?}", other)
        }
    }
}

fn serde_yaml_encode(luau: &Lua, value: LuaValue) -> LuaValueResult {
    match value {
        LuaValue::Table(table) => {
            let luau_string = std_json::json_encode(luau, LuaValue::Table(table).into_lua_multi(luau)?)?;
            let json_string: serde_json::Value = match serde_json::from_str(&luau_string) {
                Ok(s) => s,
                Err(err) => {
                    return wrap_err!("serde.yaml.encode: error parsing table to json to encode to yaml: {}", err);
                }
            };
            let yaml_string = match serde_yaml::to_string(&json_string) {
                Ok(yaml) => yaml,
                Err(err) => {
                    return wrap_err!("serde.yaml.encode: unable to parse json to yaml: {}", err);
                }
            };
            Ok(LuaValue::String(luau.create_string(&yaml_string)?))
        }
        other => {
            wrap_err!("serde.yaml.encode expected table, got: {:#?}", other)
        }
    }
}

pub fn create_yaml(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("encode", serde_yaml_encode)?
        .with_function("decode", serde_yaml_decode)?
        .with_function("readfile", move | luau: &Lua, value: LuaValue | -> LuaValueResult {
            match value {
                LuaValue::String(file_path) => {
                    let luau_value = std_fs::fs_readfile(luau, file_path.into_lua(luau)?)?;
                    serde_yaml_decode(luau, luau_value)
                },
                other => wrap_err!("yaml.readfile expected file_path (string), got {:#?}", other)
            }
        })?
        .with_function("writefile", move | luau: &Lua, value: LuaValue | -> LuaEmptyResult {
            match value {
                LuaValue::Table(data) => {
                    let file_path = match data.raw_get("path") {
                        Ok(LuaValue::String(path)) => path,
                        Ok(other) =>
                            return wrap_err!("yaml.writefile expected path to be a string, got: {:#?}", other),
                        Err(err) => {
                            return wrap_err!("yaml.writefile: unexpected error reading table: {:#?}", err);
                        }
                    };
                    let content = match data.raw_get("content") {
                        Ok(LuaValue::Table(content)) => content,
                        Ok(other) =>
                            return wrap_err!("yaml.writefile expected content to be a table, got: {:#?}", other),
                        Err(err) => {
                            return wrap_err!("yaml.writefile: unexpected error reading table: {:#?}", err);
                        }
                    };

                    let yaml_value = serde_yaml_encode(luau, LuaValue::Table(content))?;
                    let writefile_vec = vec![
                        LuaValue::String(file_path),
                        yaml_value
                    ];
                    let writefile_multivalue = LuaMultiValue::from_vec(writefile_vec);
                    std_fs::fs_writefile(luau, writefile_multivalue)
                },
                other => wrap_err!("yaml.readfile expected table to convert to yaml, got {:#?}", other)
            }
        })?
        .build_readonly()
}

fn serde_toml_decode(luau: &Lua, value: LuaValue) -> LuaValueResult {
    match value {
        LuaValue::String(data) => {
            let rust_string = data.to_string_lossy();
            match rust_string.parse::<TomlValue>() {
                Ok(toml_value) => {
                    // Convert TomlValue to Lua table
                    convert_toml_to_lua(luau, toml_value)
                }
                Err(err) => {
                    wrap_err!("serde.toml.decode: error decoding TOML: {}", err)
                }
            }
        }
        other => {
            wrap_err!("serde.toml.decode expected string, got: {:#?}", other)
        }
    }
}

fn serde_toml_encode(luau: &Lua, value: LuaValue) -> LuaValueResult {
    match value {
        LuaValue::Table(table) => {
            let rust_value = convert_lua_to_toml(luau, table)?;
            let toml_string = match toml::to_string(&rust_value) {
                Ok(toml_string) => toml_string,
                Err(err) => {
                    return wrap_err!("serde.toml.encode error: {}", err);
                }
            };
            toml_string.into_lua(luau)
        }
        other => {
            wrap_err!("serde.toml.encode expected table, got: {:#?}", other)
        }
    }
}

pub fn create_toml(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("encode", serde_toml_encode)?
        .with_function("decode", serde_toml_decode)?
        .with_function("readfile", move | luau: &Lua, value: LuaValue | -> LuaValueResult {
            match value {
                LuaValue::String(file_path) => {
                    let luau_value = std_fs::fs_readfile(luau, file_path.into_lua(luau)?)?;
                    serde_toml_decode(luau, luau_value)
                },
                other => wrap_err!("toml.readfile expected file_path (string), got {:#?}", other)
            }
        })?
        .with_function("writefile", move | luau: &Lua, value: LuaValue | -> LuaEmptyResult {
            match value {
                LuaValue::Table(data) => {
                    let file_path = match data.raw_get("path") {
                        Ok(LuaValue::String(path)) => path,
                        Ok(other) =>
                            return wrap_err!("toml.writefile expected path to be a string, got: {:#?}", other),
                        Err(err) => {
                            return wrap_err!("toml.writefile: unexpected error reading table: {:#?}", err);
                        }
                    };
                    let content = match data.raw_get("content") {
                        Ok(LuaValue::Table(content)) => content,
                        Ok(other) =>
                            return wrap_err!("toml.writefile expected content to be a table, got: {:#?}", other),
                        Err(err) => {
                            return wrap_err!("toml.writefile: unexpected error reading table: {:#?}", err);
                        }
                    };

                    let toml_value = serde_toml_encode(luau, LuaValue::Table(content))?;
                    let writefile_vec = vec![
                        LuaValue::String(file_path),
                        toml_value
                    ];
                    let writefile_multivalue = LuaMultiValue::from_vec(writefile_vec);
                    std_fs::fs_writefile(luau, writefile_multivalue)
                },
                other => wrap_err!("toml.readfile expected table to convert to toml, got {:#?}", other)
            }
        })?
        .build_readonly()
}

// Helper functions to convert between TOML and Lua types
fn convert_toml_to_lua(luau: &Lua, value: TomlValue) -> LuaValueResult {
    match value {
        TomlValue::String(s) => Ok(LuaValue::String(luau.create_string(&s)?)),
        TomlValue::Integer(i) => Ok(LuaValue::Integer(i)),
        TomlValue::Float(f) => Ok(LuaValue::Number(f)),
        TomlValue::Boolean(b) => Ok(LuaValue::Boolean(b)),
        TomlValue::Datetime(dt) => Ok(LuaValue::String(luau.create_string(dt.to_string())?)),
        TomlValue::Array(arr) => {
            let lua_table = luau.create_table()?;
            for (i, v) in arr.into_iter().enumerate() {
                lua_table.set(i + 1, convert_toml_to_lua(luau, v)?)?;
            }
            Ok(LuaValue::Table(lua_table))
        }
        TomlValue::Table(table) => {
            let lua_table = luau.create_table()?;
            for (k, v) in table.into_iter() {
                lua_table.set(k, convert_toml_to_lua(luau, v)?)?;
            }
            Ok(LuaValue::Table(lua_table))
        }
    }
}

#[allow(clippy::only_used_in_recursion)]
fn convert_lua_to_toml(luau: &Lua, table: LuaTable) -> LuaResult<TomlValue> {
    let mut toml_map = toml::map::Map::new();
    for pair in table.pairs::<LuaValue, LuaValue>() {
        let (key, value) = pair?;
        let key_str = match key {
            LuaValue::String(s) => s.to_str()?.to_string(),
            _ => return wrap_err!("serde.toml.encode: key must be a string"),
        };
        let toml_value = match value {
            LuaValue::String(s) => TomlValue::String(s.to_str()?.to_string()),
            LuaValue::Integer(i) => TomlValue::Integer(i),
            LuaValue::Number(n) => TomlValue::Float(n),
            LuaValue::Boolean(b) => TomlValue::Boolean(b),
            LuaValue::Table(t) => convert_lua_to_toml(luau, t)?,
            _ => return wrap_err!("serde.toml.encode: unsupported Lua type"),
        };
        toml_map.insert(key_str, toml_value);
    }
    Ok(TomlValue::Table(toml_map))
}

fn serde_base64_encode(luau: &Lua, data: LuaValue) -> LuaValueResult {
    match data {
        LuaValue::Buffer(buffy) => {
            let rust_vec_u8: Vec<u8> = buffy.to_vec();
            let encoded_string = base64::encode(rust_vec_u8);
            encoded_string.into_lua(luau)
        },
        LuaValue::String(_data) => {
            wrap_err!("serde.base64.encode: got string, please pass buffer instead")
        },
        other => {
            wrap_err!("serde.base64.encode expected buffer, got: {:#?}", other)
        }
    }
}

fn serde_base64_decode(luau: &Lua, data: LuaValue) -> LuaValueResult {
    match data {
        LuaValue::String(data) => {
            let rust_string = data.to_string_lossy();
            let decoded_data = match base64::decode(rust_string) {
                Ok(data) => data,
                Err(err) => {
                    return wrap_err!("serde.base64.encode: error decoding base64: {}", err);
                }
            };
            let buffy = luau.create_buffer(decoded_data)?;
            Ok(LuaValue::Buffer(buffy))
        },
        other => {
            wrap_err!("serde.base64.decode expected string, got: {:#?}", other)
        }
    }
}

pub fn create_base64(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("encode", serde_base64_encode)?
        .with_function("decode", serde_base64_decode)?
        .build_readonly()
}

fn serde_hex_encode(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let data = match value {
        LuaValue::Buffer(buffy) => {
            buffy.to_vec()
        },
        LuaValue::String(data) => {
            let data = data.as_bytes();
            data.to_vec()
        }
        other => {
            return wrap_err!("serde.hex.encode: expected buffer (or string), got: {:?}", other);
        }
    };
    let rust_string = hex::encode(data);
    Ok(LuaValue::String(
        luau.create_string(rust_string)?
    ))
}

fn serde_hex_decode(luau: &Lua, value: LuaValue) -> LuaValueResult {
    match value {
        LuaValue::String(data) => {
            let rust_string = match data.to_str() {
                Ok(data) => data.to_string(),
                Err(_err) => {
                    return wrap_err!("serde.hex.decode: encoded string contains invalid UTF-8 data; strings passed to this function should already be encoded and shouldn't be able to contain invalid UTF-8 characters")
                }
            };
            let decoded_hex = match hex::decode(rust_string) {
                Ok(decoded) => decoded,
                Err(err) => {
                    return wrap_err!("serde.hex.decode: unable to decode hex string: {}", err);
                }
            };
            let luau_hex_buffy = luau.create_buffer(decoded_hex)?;
            Ok(LuaValue::Buffer(luau_hex_buffy))
        },
        other => {
           wrap_err!("serde.hex.decode: expected string, got: {:?}", other) 
        }
    }
}

pub fn create_hex(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("encode", serde_hex_encode)?
        .with_function("decode", serde_hex_decode)?
        .build_readonly()
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_value("base64", LuaValue::Table(
            create_base64(luau)?
        ))?
        .with_value("json", LuaValue::Table(std_json::create(luau)?))?
        .with_value("toml", LuaValue::Table(create_toml(luau)?))?
        .with_value("yaml", LuaValue::Table(create_yaml(luau)?))?
        .with_value("hex", LuaValue::Table(create_hex(luau)?))?
        .build_readonly()
}
