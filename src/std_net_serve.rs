#[allow(unused_imports)]
use crate::{colors, std_json, table_helpers::TableBuilder, LuaValueResult};
use mlua::prelude::*;
use regex::Regex;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

fn server_serve(luau: &Lua, serve_config: LuaValue) -> LuaValueResult {
    let config = match serve_config {
        LuaValue::Table(config) => config,
        other => {
            return wrap_err!("server.serve expected ServeConfig table (with fields address, port, handler, etc.), got: {:#?}", other);
        }
    };

    let address: String = match config.raw_get("address") {
        Ok(address) => address,
        Err(err) => {
            return wrap_err!("server.serve expected an address (string), got an error: {}", err);
        }
    };

    let port: String = match config.raw_get("port") {
        Ok(port) => match port {
            LuaValue::String(port) => port.to_string_lossy(),
            LuaValue::Integer(port) => port.to_string(),
            other => {
                return wrap_err!("server.serve expected a port, got: {:#?}", other);
            }
        },
        Err(err) => {
            return wrap_err!("server.serve expected a port (string), got an error: {}", err);
        }
    };

    let handler_function = match config.raw_get("handler") {
        Ok(LuaValue::Function(f)) => f,
        Ok(LuaValue::Table(_handles)) => todo!(),
        Ok(other) => {
            return wrap_err!("server.serve expected handler to be a function, got: {:#?}", other);
        }
        Err(err) => {
            return wrap_err!("server.serve expected some handler, got an error: {}", err);
        }
    };

    let address_port = format!("{}:{}", address, port);
    let listener = match TcpListener::bind(&address_port) {
        Ok(listener) => listener,
        Err(err) => {
            return wrap_err!("server.serve: failed to bind to {} with error: {}", address_port, err);
        }
    };

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                match handle_client(&mut stream, handler_function.clone(), luau) {
                    Ok(_client) => {}
                    Err(err) => {
                        return wrap_err!("server.serve: failed to handle client: {}", err);
                    }
                }
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }

    Ok(LuaValue::Nil)
}

fn handle_client(stream: &mut TcpStream, handler_function: LuaFunction, luau: &Lua) -> LuaValueResult {
    let mut buffer = [0; 512];
    match stream.read_exact(&mut buffer) {
        Ok(_) => {
            let request = String::from_utf8_lossy(&buffer);

            let lines: Vec<&str> = request.lines().collect();

            let first_line_re = Regex::new(r"^(?P<method>\w+)\s+(?P<path>\S+)\s+HTTP/1\.[01]$").unwrap();
            let captures = match first_line_re.captures(lines[0]) {
                Some(captures) => captures,
                None => return wrap_err!("server.serve: Failed to parse request line: {}", lines[0]),
            };

            let method = match captures.name("method") {
                Some(method) => method.as_str(),
                None => return wrap_err!("server.serve: Failed to extract method: {}", lines[0]),
            };
            let path = match captures.name("path") {
                Some(path) => path.as_str(),
                None => return wrap_err!("server.serve: Failed to extract path: {}", lines[0]),
            };

            // Collect headers
            let headers_table = luau.create_table()?;
            let mut i = 1;
            while i < lines.len() && !lines[i].is_empty() {
                let mut header_line = lines[i].to_string();
                while i + 1 < lines.len() && lines[i + 1].starts_with(' ') {
                    i += 1;
                    header_line.push_str(&format!("\r\n{}", lines[i]));
                }

                if let Some((key, value)) = header_line.split_once(": ") {
                    headers_table.raw_set(key, value)?;
                } else {
                    // sometimes we get fails to parse header line and IDK they look like half cut off words and stuff
                    // by that time we usually got some headers so let's just break out of it
                    // if we continue instead of break we basically hit a spinlock of Failed to parse header line stuff
                    break;
                    // return wrap_err!("server.serve: Failed to parse header line: {}", header_line);
                }
                i += 1;
            }

            // Body starts after the empty line following headers
            let body = if i < lines.len() - 1 {
                lines[i + 1..].join("\n")
            } else {
                String::new()
            };

            let serve_request_info = TableBuilder::create(luau)?
                .with_value("method", method)?
                .with_value("path", path)?
                .with_value("headers", headers_table)?
                .with_value("body", body)?
                .with_value("raw_text", request.clone())?
                .build_readonly()?;

            let serve_response: LuaTable = match handler_function.call(serve_request_info) {
                Ok(res) => match res {
                    LuaValue::Table(table) => table,
                    other => return wrap_err!("server.serve: handler_function should return a table, got: {:#?}", other),
                },
                Err(err) => return wrap_err!("server.serve: handler_function call failed with error: {}", err),
            };

            let status_code: String = match serve_response.raw_get("status_code") {
                Ok(status) => status,
                Err(err) => return wrap_err!("ServeResponse table missing 'status_code': {}", err),
            };

            let response_type: String = match serve_response.raw_get("response_type") {
                Ok(response_type) => {
                    if let LuaValue::String(response) = response_type {
                        let response = response.to_string_lossy();
                        match response.to_lowercase().as_str() {
                            "text" => "text/plain; charset=utf-8".to_string(),
                            "html" => "text/html; charset=utf-8".to_string(),
                            "json" => "application/json".to_string(),
                            "xml"  => "application/xml".to_string(),
                            "css"  => "text/css".to_string(),
                            "binary" => "application/octet-stream".to_string(),
                            other => other.to_string()
                        }
                    } else {
                        return wrap_err!("ServeResponse expected response_type to be a string, got: {:#?}", response_type);
                    }
                },
                Err(err) => return wrap_err!("ServeResponse table missing 'response_type': {}", err),
            };

            let mut buffer_body: Option<Vec<u8>> = None;
            let body: String = match serve_response.raw_get("body") {
                Ok(LuaValue::String(body)) => body.to_string_lossy(),
                Ok(LuaValue::Buffer(buff)) => {
                    let bytes = buff.to_vec();
                    buffer_body = Some(bytes);
                    String::from("")
                },
                Ok(other) => {
                    return wrap_err!("Expected body to be a string (or buffer), got: {:#?}", other);
                }
                Err(err) => return wrap_err!("ServeResponse table missing 'body': {}", err),
            };

            let headers: Option<LuaTable> = serve_response.raw_get("headers").ok();
            let cookies: Option<LuaTable> = serve_response.raw_get("cookies").ok();
            let http_version: Option<String> = serve_response.raw_get("http_version").ok();
            let reason_phrase: Option<String> = serve_response.raw_get("reason_phrase").ok();
            let redirect_url: Option<String> = serve_response.raw_get("redirect_url").ok();

            let http_version = http_version.unwrap_or_else(|| "HTTP/1.1".to_string());
            let reason_phrase = reason_phrase.unwrap_or_else(|| "OK".to_string());

            // Construct headers and cookies
            let mut additional_headers = String::new();
            if let Some(headers_table) = headers {
                for pair in headers_table.pairs::<LuaString, LuaString>().flatten() {
                    let (key, value) = pair;
                    additional_headers.push_str(&format!("{}: {}\r\n", key.to_str()?, value.to_str()?));
                }
            }
            if let Some(cookies_table) = cookies {
                for pair in cookies_table.pairs::<LuaString, LuaString>().flatten() {
                    let (key, value) = pair;
                    additional_headers.push_str(&format!("Set-Cookie: {}={}\r\n", key.to_str()?, value.to_str()?));
                }
            }
            if let Some(url) = redirect_url {
                additional_headers.push_str(&format!("Location: {}\r\n", url));
            }

            // Respond with the specified content
            let response = format!("{} {} {}\r\nContent-Type: {}\r\n{}Content-Length: {}\r\n\r\n{}", 
                http_version, status_code, reason_phrase, response_type, additional_headers, body.len(), body);
            
            // now we actually send and write to stream
            // if someone passed in body = somebuffer then we have to write the buffer to the stream separately
            match buffer_body {
                Some(bytes) => {
                    match stream.write_all(response.as_bytes()).and_then(|_| stream.write_all(&bytes) ) {
                        Ok(_) => match stream.flush() {
                            Ok(_) => Ok(LuaValue::Nil),
                            Err(err) => wrap_err!("Failed to flush stream: {}", err),
                        },
                        Err(err) => wrap_err!("Failed to write response: {}", err),
                    }
                }
                None => {
                    match stream.write_all(response.as_bytes()) {
                        Ok(_) => match stream.flush() {
                            Ok(_) => Ok(LuaValue::Nil),
                            Err(err) => wrap_err!("Failed to flush stream: {}", err),
                        },
                        Err(err) => wrap_err!("Failed to write response: {}", err),
                    }
                }
            }
            
        }
        Err(err) => wrap_err!("server.serve: Failed to read from stream: {}", err),
    }
}


pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("serve", server_serve)?
        .build_readonly()
}