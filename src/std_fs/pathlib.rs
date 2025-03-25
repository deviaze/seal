use mlua::prelude::*;
use std::io;
use std::fs;
use std::path::{self, PathBuf, Path};
use crate::{colors, LuaValueResult, wrap_err, table_helpers::TableBuilder, std_fs::fs_exists};

fn fs_path_join(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let mut path = PathBuf::new();
    let mut index = 0;
    while let Some(component) = multivalue.pop_front() {
        let component = match component {
            LuaValue::String(component) => {
                // passing a path starting with / or \ into path.push replaces the current path
                // path.join("./src", "/main.luau") should not return "/main.luau"
                // this strips any of those for better ux
                let separators_to_trim = ['/', '\\']; 
                let component = component.to_string_lossy();
                if index > 0 { // dont remove / from absolute paths that start in / on purpose
                    index += 1;
                    component.trim_start_matches(separators_to_trim).to_string()
                } else {
                    component
                }
            },
            other => {
                return wrap_err!("path.join expected path to be a string, got: {:#?}", other);
            }
        };
        path.push(component);
    }
    Ok(LuaValue::String(luau.create_string(path.to_string_lossy().to_string())?))
}

fn fs_path_canonicalize(luau: &Lua, path: LuaValue) -> LuaValueResult {
    let path = match path {
        LuaValue::String(path) => path.to_string_lossy(),
        other => {
            return wrap_err!("path.canonicalize(path) expected path to be a string, got: {:#?}", other);
        }
    };

    match fs::canonicalize(&path) {
        Ok(canonical_path) => {
            Ok(LuaValue::String(luau.create_string(canonical_path.to_string_lossy().to_string())?))
        },
        Err(err) => {
            match err.kind() {
                io::ErrorKind::NotFound => {
                    if !path.starts_with(".") && !path.starts_with("..") {
                        wrap_err!("path.canonicalize: requested path '{}' doesn't exist on the filesystem. Did you forget to use a relative path (starting with . or .. like \"./libs/helper.luau\")?", path)
                    } else {
                        wrap_err!("path.canonicalize: requested path '{}' doesn't exist on the filesystem. Consider using path.absolutize if your path doesn't exist yet.", path)
                    }
                },
                _ => {
                    wrap_err!("path.canonicalize: error canonicalizing path '{}': {}", path, err)
                }
            }
        }
    }
}

fn fs_path_absolutize(luau: &Lua, path: LuaValue) -> LuaValueResult {
    let path = match path {
        LuaValue::String(path) => path.to_string_lossy(),
        other => {
            return wrap_err!("path.absolutize(path) expected path to be a string, got: {:#?}", other);
        }
    };

    match path::absolute(&path) {
        Ok(path) => {
            Ok(LuaValue::String(luau.create_string(path.to_string_lossy().to_string())?))
        },
        Err(err) => {
            wrap_err!("path.absolutize: error getting absolute path: {}", err)
        }
    }
}

fn fs_path_parent(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let requested_path = match multivalue.pop_front() {
        Some(path) => {
            match path {
                LuaValue::String(path) => path.to_string_lossy(),
                other => {
                    return wrap_err!("path.parent(path: string, n: number?) expected path to be a string, got: {:#?}", other);
                }
            }
        },
        None => {
            return wrap_err!("path.parent(path) expected path to be a string but was called with zero arguments")
        }
    };

    let n_parents = match multivalue.pop_front() {
        Some(n) => {
            match n {
                LuaValue::Integer(n) => n,
                LuaValue::Number(f) => {
                    return wrap_err!("path.parent(path: string, n: number?) expected n to be a whole number/integer, got float {}", f);
                }
                LuaNil => 1,
                other => {
                    return wrap_err!("path.parent(path: string, n: number?) expected n to be a number or nil, got: {:#?}", other)
                }
            }
        },
        None => 1
    };

    let path = Path::new(&requested_path);
    let mut current_path = path;
    for _ in 0..n_parents {
        match current_path.parent() {
            Some(parent) => {
                current_path = parent;
            },
            None => {
                return Ok(LuaNil);
            }
        }
    }
    
    Ok(LuaValue::String(luau.create_string(current_path.to_string_lossy().to_string())?))
}

fn fs_path_child(luau: &Lua, path: LuaValue) -> LuaValueResult {
    let requested_path = match path {
        LuaValue::String(path) => path.to_string_lossy(),
        other => {
            return wrap_err!("path.child(path) expected path to be a string, got: {:#?}", other);
        }
    };

    let path = Path::new(&requested_path);
    match path.file_name() {
        Some(name) => {
            let name = name.to_string_lossy().to_string();
            Ok(LuaValue::String(luau.create_string(&name)?))
        },
        None => {
            Ok(LuaNil)
        }
    }
}

fn fs_path_home(luau: &Lua, _value: LuaValue) -> LuaValueResult {
    #[allow(deprecated)] // env::home_dir() is undeprecated now
    if let Some(home_dir) = std::env::home_dir() {
        let home_dir = home_dir.to_string_lossy().to_string();
        Ok(LuaValue::String(luau.create_string(&home_dir)?))
    } else {
        Ok(LuaNil)
    }
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("join", fs_path_join)?
        .with_function("exists", fs_exists)?
        .with_function("canonicalize", fs_path_canonicalize)?
        .with_function("absolutize", fs_path_absolutize)?
        .with_function("parent", fs_path_parent)?
        .with_function("child", fs_path_child)?
        .with_function("home", fs_path_home)?
        .build_readonly()
}