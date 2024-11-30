use ureq::{self, Error as UreqError};
use mlua::prelude::*;

use crate::{std_io_colors as colors, std_json};
use crate::{table_helpers::TableBuilder, LuaValueResult};

pub fn http_get(luau: &Lua, get_config: LuaValue) -> LuaValueResult {
    match get_config {
        LuaValue::String(url) => {
            let url = url.to_str()?.to_string();
            match ureq::get(&url).call() {
                Ok(mut response) => {
                    let body = response.body_mut().read_to_string().into_lua_err()?;
                    let result = TableBuilder::create(luau)?
                        .with_value("ok", true)?
                        .with_value("body", body.clone())?
                        .with_function("decode", {
                            move | luau: &Lua, _: LuaMultiValue | {
                                std_json::json_decode(luau, body.to_owned())
                            }
                        })?
                        .build_readonly()?;
                    Ok(LuaValue::Table(result))
                },
                Err(UreqError::StatusCode(code)) => {
                    let err_message = format!("HTTP error: {}", code);
                    let result = luau.create_table()?;
                    result.set("ok", false)?;
                    result.set("err", err_message)?;
                    Ok(LuaValue::Table(result))
                },
                Err(_) => {
                    wrap_err!("Some sort of HTTP I/O/transport/network error occurred...")
                }
            }
        },
        LuaValue::Table(config) => {
            let url: String = {
                match config.get("url")? {
                    LuaValue::String(url) => url.to_str()?.to_string(),
                    LuaValue::Nil => {
                        return wrap_err!("net.GetConfig missing field url")
                    },
                    other => {
                        return wrap_err!("net.get GetConfig expected url to be a string, got: {:?}", other)
                    }
                }
            };
            // let mut get_builder = ureq::get(&url);
            let mut get_builder = ureq::get(&url);

            if let LuaValue::Table(headers_table) = config.get("headers")? {
                for pair in headers_table.pairs::<String, String>() {
                    let (key, value) = pair?;
                    get_builder = get_builder.header(key, value);
                }
            };

            if let LuaValue::Table(headers_table) = config.get("params")? {
                for pair in headers_table.pairs::<String, String>() {
                    let (key, value) = pair?;
                    get_builder = get_builder.query(key, value);
                }
            };

            let body: Option<String> = {
                match config.get("body")? {
                    LuaValue::String(body) => Some(body.to_str()?.to_string()),
                    LuaValue::Table(body_table) => {
                        get_builder = get_builder.header("Content-Type", "application/json");
                        Some(std_json::json_encode_data(luau, LuaValue::Table(body_table))?)
                    },
                    LuaValue::Nil => None,
                    other => {
                        return wrap_err!("net.get GetOptions.body expected table (to serialize as json) or string, got: {:?}", other)
                    }
                }
            };

            let send_result = {
                if let Some(body) = body {
                    get_builder.force_send_body().send(body)
                } else {
                    get_builder.call()
                }
            };

            match send_result {
                Ok(mut result) => {
                    let body = result.body_mut().read_to_string().unwrap_or(String::from(""));
                    
                    let json_decode_body = {
                        let body_clone = body.clone();
                        move |luau: &Lua, _: LuaMultiValue| {
                            match std_json::json_decode(luau, body_clone.to_owned()) {
                                Ok(response) => Ok(response),
                                Err(err) => {
                                    wrap_err!("NetResponse:decode() unable to decode response.body to json: {}", err)
                                }
                            }
                        }
                    };
                    let result = TableBuilder::create(luau)?
                        .with_value("ok", true)?
                        .with_value("body", body)?
                        .with_function("decode", json_decode_body.to_owned())?
                        .with_function("unwrap", json_decode_body.to_owned())?
                        .build_readonly()?;
                    Ok(LuaValue::Table(result))
                },
                Err(err) => {
                    let err_result = TableBuilder::create(luau)?
                        .with_value("ok", false)?
                        .with_value("err", err.to_string())?
                        .with_function("unwrap", |_luau: &Lua, mut default: LuaMultiValue| {
                            let response = default.pop_front().unwrap();
                            let default = default.pop_back();
                            match default {
                                Some(LuaValue::Nil) => {
                                    wrap_err!("net.get: attempted to unwrap an erred request; note: default argument provided but was nil. Erred request: {:#?}", response)
                                },
                                None => {
                                    wrap_err!("net.get: attempted to unwrap an erred request without default argument. Erred request: {:#?}", response)
                                },
                                Some(other) => {
                                    Ok(other)
                                }
                            }
                        })?
                        .build_readonly()?;
                    Ok(LuaValue::Table(err_result))
                }
            }
        }
        other => {
           wrap_err!("net.get expected url: string or GetOptions, got {:?}", other)
        }
    }
}

pub fn http_post(luau: &Lua, post_config: LuaValue) -> LuaValueResult {
    match post_config {
        LuaValue::Table(config) => {
            let url: String = {
                match config.get("url")? {
                    LuaValue::String(url) => url.to_string_lossy(),
                    LuaValue::Nil => {
                        return wrap_err!("net.post: PostConfig missing url field")
                    },
                    other => {
                        return wrap_err!("net.post: PostConfig expected url to be a string, got: {:?}", other)
                    }
                }
            };
            // let mut get_builder = ureq::get(&url);
            let mut post_builder = ureq::post(&url);

            if let LuaValue::Table(headers_table) = config.get("headers")? {
                for pair in headers_table.pairs::<String, String>() {
                    let (key, value) = pair?;
                    post_builder = post_builder.header(key, value);
                }
            };

            if let LuaValue::Table(headers_table) = config.get("params")? {
                for pair in headers_table.pairs::<String, String>() {
                    let (key, value) = pair?;
                    post_builder = post_builder.query(key, value);
                }
            };

            let body = {
                match config.get("body")? {
                    LuaValue::String(body) => body.to_str()?.to_string(),
                    LuaValue::Table(body_table) => {
                        post_builder = post_builder.header("Content-Type", "application/json");
                        std_json::json_encode_data(luau, LuaValue::Table(body_table))?
                    },
                    other => {
                        return wrap_err!("net.post PostOptions.body expected table (to serialize as json) or string, got: {:?}", other)
                    }
                }
            };

            match post_builder.send(body) {
                Ok(mut result) => {
                    let body = result.body_mut().read_to_string().unwrap_or(String::from(""));
                    let json_decode_body = {
                        let body_clone = body.clone();
                        move |luau: &Lua, _: LuaMultiValue| -> LuaValueResult {
                            std_json::json_decode(luau, body_clone.to_owned())
                        }
                    };
                    let result = TableBuilder::create(luau)?
                        .with_value("ok", true)?
                        .with_value("body", body.clone())?
                        .with_function("decode", json_decode_body.to_owned())?
                        .with_function("unwrap", json_decode_body.to_owned())?
                        .build_readonly()?;
                    Ok(LuaValue::Table(result))
                },
                Err(err) => {
                    let err_result = TableBuilder::create(luau)?
                        .with_value("ok", false)?
                        .with_value("err", err.to_string())?
                        .with_function("unwrap", |_luau: &Lua, default: LuaValue| {
                            match default {
                                LuaValue::Nil => {
                                    wrap_err!("net.post: attempted to unwrap an erred request without default argument")
                                },
                                other => {
                                    Ok(other)
                                }
                            }
                        })?
                        .build_readonly()?;
                    Ok(LuaValue::Table(err_result))
                }
            }
        }
        other => {
           wrap_err!("net.post expected PostConfig, got {:?}", other)
        }
    }
}

fn http_put(luau: &Lua, put_config: LuaValue) -> LuaValueResult {
    match put_config {
        LuaValue::Table(config) => {
            let url: String = {
                match config.get("url")? {
                    LuaValue::String(url) => url.to_string_lossy(),
                    LuaValue::Nil => {
                        return wrap_err!("net.put: PutConfig missing url field")
                    },
                    other => {
                        return wrap_err!("net.put: PutConfig expected url to be a string, got: {:?}", other)
                    }
                }
            };
            // let mut get_builder = ureq::get(&url);
            let mut put_builder = ureq::put(&url);

            if let LuaValue::Table(headers_table) = config.get("headers")? {
                for pair in headers_table.pairs::<String, String>() {
                    let (key, value) = pair?;
                    put_builder = put_builder.header(key, value);
                }
            };

            if let LuaValue::Table(headers_table) = config.get("params")? {
                for pair in headers_table.pairs::<String, String>() {
                    let (key, value) = pair?;
                    put_builder = put_builder.query(key, value);
                }
            };

            let body = {
                match config.get("body")? {
                    LuaValue::String(body) => body.to_str()?.to_string(),
                    LuaValue::Table(body_table) => {
                        put_builder = put_builder.header("Content-Type", "application/json");
                        std_json::json_encode_data(luau, LuaValue::Table(body_table))?
                    },
                    other => {
                        return wrap_err!("net.get PutOptions.body expected table (to serialize as json) or string, got: {:?}", other)
                    }
                }
            };

            match put_builder.send(body) {
                Ok(mut result) => {
                    let body = result.body_mut().read_to_string().unwrap_or(String::from(""));
                    let json_decode_body = {
                        let body_clone = body.clone();
                        move |luau: &Lua, _: LuaMultiValue| -> LuaValueResult {
                            std_json::json_decode(luau, body_clone.to_owned())
                        }
                    };
                    let result = TableBuilder::create(luau)?
                        .with_value("ok", true)?
                        .with_value("body", body.clone())?
                        .with_function("decode", json_decode_body.to_owned())?
                        .with_function("unwrap", json_decode_body.to_owned())?
                        .build_readonly()?;
                    Ok(LuaValue::Table(result))
                },
                Err(err) => {
                    let err_result = TableBuilder::create(luau)?
                        .with_value("ok", false)?
                        .with_value("err", err.to_string())?
                        .with_function("unwrap", |_luau: &Lua, default: LuaValue| {
                            match default {
                                LuaValue::Nil => {
                                    wrap_err!("net.put: attempted to unwrap an erred request without default argument")
                                },
                                other => {
                                    Ok(other)
                                }
                            }
                        })?
                        .build_readonly()?;
                    Ok(LuaValue::Table(err_result))
                }
            }
        }
        other => {
           wrap_err!("net.put expected PutConfig, got {:?}", other)
        }
    }
}

fn http_patch(luau: &Lua, patch_config: LuaValue) -> LuaValueResult {
    match patch_config {
        LuaValue::Table(config) => {
            let url: String = {
                match config.get("url")? {
                    LuaValue::String(url) => url.to_string_lossy(),
                    LuaValue::Nil => {
                        return wrap_err!("net.request: PATCH: PatchConfig missing url field")
                    },
                    other => {
                        return wrap_err!("net.request: PATCH: PatchConfig.url expected to be a string, got: {:?}", other)
                    }
                }
            };
            // let mut get_builder = ureq::get(&url);
            let mut patch_builder = ureq::patch(&url);

            if let LuaValue::Table(headers_table) = config.get("headers")? {
                for pair in headers_table.pairs::<String, String>() {
                    let (key, value) = pair?;
                    patch_builder = patch_builder.header(key, value);
                }
            };

            if let LuaValue::Table(headers_table) = config.get("params")? {
                for pair in headers_table.pairs::<String, String>() {
                    let (key, value) = pair?;
                    patch_builder = patch_builder.query(key, value);
                }
            };

            let body = {
                match config.get("body")? {
                    LuaValue::String(body) => body.to_str()?.to_string(),
                    LuaValue::Table(body_table) => {
                        patch_builder = patch_builder.header("Content-Type", "application/json");
                        std_json::json_encode_data(luau, LuaValue::Table(body_table))?
                    },
                    other => {
                        return wrap_err!("net.request: PATCH: PatchOptions.body expected to be table (to serialize as json) or string, got: {:?}", other)
                    }
                }
            };

            match patch_builder.send(body) {
                Ok(mut result) => {
                    let body = result.body_mut().read_to_string().unwrap_or(String::from(""));
                    let json_decode_body = {
                        let body_clone = body.clone();
                        move |luau: &Lua, _: LuaMultiValue| -> LuaValueResult {
                            std_json::json_decode(luau, body_clone.to_owned())
                        }
                    };
                    let result = TableBuilder::create(luau)?
                        .with_value("ok", true)?
                        .with_value("body", body.clone())?
                        .with_function("decode", json_decode_body.to_owned())?
                        .with_function("unwrap", json_decode_body.to_owned())?
                        .build_readonly()?;
                    Ok(LuaValue::Table(result))
                },
                Err(err) => {
                    let err_result = TableBuilder::create(luau)?
                        .with_value("ok", false)?
                        .with_value("err", err.to_string())?
                        .with_function("unwrap", |_luau: &Lua, default: LuaValue| {
                            match default {
                                LuaValue::Nil => {
                                    wrap_err!("net.request: PATCH: attempted to unwrap an erred request without default argument")
                                },
                                other => {
                                    Ok(other)
                                }
                            }
                        })?
                        .build_readonly()?;
                    Ok(LuaValue::Table(err_result))
                }
            }
        }
        other => {
           wrap_err!("net.request: PATCH: expected PatchConfig, got {:?}", other)
        }
    }
}

fn http_delete(luau: &Lua, delete_config: LuaValue) -> LuaValueResult {
    match delete_config {
        LuaValue::Table(config) => {
            let url: String = {
                match config.get("url")? {
                    LuaValue::String(url) => url.to_string_lossy(),
                    LuaValue::Nil => {
                        return wrap_err!("net.request: DELETE: DeleteConfig missing url field")
                    },
                    other => {
                        return wrap_err!("net.request: DELETE: DeleteConfig.url expected to be a string, got: {:?}", other)
                    }
                }
            };
            // let mut get_builder = ureq::get(&url);
            let mut delete_builder = ureq::delete(&url);

            if let LuaValue::Table(headers_table) = config.get("headers")? {
                for pair in headers_table.pairs::<String, String>() {
                    let (key, value) = pair?;
                    delete_builder = delete_builder.header(key, value);
                }
            };

            if let LuaValue::Table(headers_table) = config.get("params")? {
                for pair in headers_table.pairs::<String, String>() {
                    let (key, value) = pair?;
                    delete_builder = delete_builder.query(key, value);
                }
            };

            match delete_builder.call() {
                Ok(mut result) => {
                    let body = result.body_mut().read_to_string().unwrap_or(String::from(""));
                    let json_decode_body = {
                        let body_clone = body.clone();
                        move |luau: &Lua, _: LuaMultiValue| -> LuaValueResult {
                            std_json::json_decode(luau, body_clone.to_owned())
                        }
                    };
                    let result = TableBuilder::create(luau)?
                        .with_value("ok", true)?
                        .with_value("body", body.clone())?
                        .with_function("decode", json_decode_body.to_owned())?
                        .with_function("unwrap", json_decode_body.to_owned())?
                        .build_readonly()?;
                    Ok(LuaValue::Table(result))
                },
                Err(err) => {
                    let err_result = TableBuilder::create(luau)?
                        .with_value("ok", false)?
                        .with_value("err", err.to_string())?
                        .with_function("unwrap", |_luau: &Lua, default: LuaValue| {
                            match default {
                                LuaValue::Nil => {
                                    wrap_err!("net.request: PATCH: attempted to unwrap an erred request without default argument")
                                },
                                other => {
                                    Ok(other)
                                }
                            }
                        })?
                        .build_readonly()?;
                    Ok(LuaValue::Table(err_result))
                }
            }
        }
        other => {
           wrap_err!("net.request: DELETE: expected DeleteConfig, got {:?}", other)
        }
    }
}

pub fn http_request(luau: &Lua, request_options: LuaValue) -> LuaValueResult {
    match request_options {
        LuaValue::Table(options) => {
            let method: String = match options.raw_get("method") {
                Ok(LuaValue::String(method)) => {
                    let method = method.to_string_lossy().to_uppercase();
                    match method.as_str() {
                        "GET" | "POST" | "PUT" | "PATCH"| "DELETE" => method,
                        other => {
                            return wrap_err!("net.request expected `method` to be a valid HTTP verb, got: {}", other);
                        },
                    }
                },                
                Ok(other) => {
                    return wrap_err!(r#"net.request expected options.method (string "GET" | "POST" | "PUT" | "PATCH" | "DELETE),  got {:?}"#, other)
                },
                Err(err) => {
                    return wrap_err!("net.request expected options.method to be an HTTP method verb (string), got an error: {}", err);
                }
            };
            match method.as_str() {
                "GET" => http_get(luau, LuaValue::Table(options)),
                "POST" => http_post(luau, LuaValue::Table(options)),
                "PUT" => http_put(luau, LuaValue::Table(options)),
                "PATCH" => http_patch(luau, LuaValue::Table(options)),
                "DELETE" => http_delete(luau, LuaValue::Table(options)),
                other => {
                    todo!("http verb {} not yet implemented", other)
                }
            }
        },
        other => {
            wrap_err!("net.request expected table RequestOptions, got: {:?}", other)
        }
    }
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("get", http_get)?
        .with_function("post", http_post)?
        .with_function("request", http_request)?
        .build_readonly()
}