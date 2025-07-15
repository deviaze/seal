use mluau::prelude::*;
use std::collections::VecDeque;
use std::io;
use std::fs;
use std::path::{self, Path};
use crate::prelude::*;

use super::validate_path_without_checking_fs;

fn trim_path(path: &str) -> &str {
    path.trim_matches(['/', '\\'])
}

pub fn path_join(mut components: VecDeque<String>) -> String {
    // we don't want to use PathBuf::join because it gives unexpected behavior ('./tests/luau/std\crypt\hash.luau' lol)
    let first_component = components.pop_front().unwrap_or_default();
    // we want to use forward slash paths as much as possible, and only use \ paths if we're dealing with
    // absolute paths on windows
    // but if a user passes ".\" as their first component they obviously want \ paths
    let path_sep = match first_component.as_str() {
        "./" | "../" | "." | "" | "/" => "/", // unix style '/' (default) that works on windows, linux, unix, macos, etc.
        ".\\" | "..\\" => "\\", // windows style '\' for windows absolute paths
        // stupid windows absolute path edge cases
        component if component.ends_with(':') => "\\", // handle drive letters like "C:"
        component if component.starts_with(r"\\") => "\\", // absolute paths starting with backslash (\\wsl\\)
        component if component.contains(':') => "\\",     // paths with drive letters (e.g., "C:\")
        _ => "/", // probably a path.join("dogs", "cats") partial path, default to /
    };

    let mut result = String::new();

    // Avoid stripping unix root `/` on first component
    result.push_str(if first_component.starts_with('/') {
        if first_component.len() > 1 {
            first_component.trim_end_matches(['/', '\\'])
        } else {
            "/"
        }
    } else if first_component.starts_with(r"\\") {
        // avoid stripping windows root `\\` (unc paths like \\?\C:\Users\sealey...) on first component
        first_component.trim_end_matches(['/', '\\'])
    } else {
        trim_path(&first_component)
    });

    for component in components {
        let trimmed_component = trim_path(&component);
        if !trimmed_component.is_empty() {
            result.push_str(path_sep);
            result.push_str(trimmed_component);
        }
    }

    result
}


/// fixes `./tests/luau/std\fs\pathlib_join.luau` nonsense on windows
pub fn normalize_path(path: &str) -> String {
    // Check if the path is a Windows absolute path (drive letter or UNC path)
    let is_windows_absolute = path.starts_with(r"\\") || path.chars().nth(1) == Some(':');

    // Determine the separator to use
    let separator = if is_windows_absolute { '\\' } else { '/' };

    // dont allocate strings multiple times
    let mut normalized = String::with_capacity(path.len());
    let mut previous_was_separator = false;

    for (index, c) in path.chars().enumerate() {
        if c == '/' || c == '\\' {
            if previous_was_separator {
                if index == 1 && is_windows_absolute && path.starts_with(r"\\") {
                    // dont strip unc prefixes on windows :cry:
                } else {
                    continue;
                }
            }
            normalized.push(separator);
            previous_was_separator = true;
        } else {
            normalized.push(c);
            previous_was_separator = false;
        }
    }

    normalized
}

fn fs_path_join(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "fs.path.join(...string)";
    let mut components = VecDeque::new();
    let mut index = 0;
    while let Some(component) = multivalue.pop_front() {
        index += 1;
        let component_string = match component {
            LuaValue::String(component) => {
                let Ok(component) = component.to_str() else {
                    return wrap_err!("{}: component at index {} is invalid utf-8", function_name, index);
                };
                component.to_string()
            },
            other => {
                return wrap_err!("{} expected component at index {} to be a string, got: {:?}", function_name, index, other);
            }
        };
        components.push_back(component_string);
    }
    let result = path_join(components);
    Ok(LuaValue::String(luau.create_string(&result)?))
}

fn fs_path_normalize(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "path.normalize(path: string)";
    let requested_path = match value {
        LuaValue::String(s) => {
            validate_path_without_checking_fs(&s, function_name)?
        },
        other => {
            return wrap_err!("{} expected path to be a string, got: {:?}", function_name, other);
        }
    };
    ok_string(normalize_path(&requested_path), luau)
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
            #[allow(unused_mut, reason = "needs to be mut on windows")]
            let mut canonical_path = canonical_path.to_string_lossy().to_string();
            #[cfg(windows)]
            {
                // very cool unc paths windows
                canonical_path = canonical_path.replace(r"\\?\", "");
            }
            Ok(LuaValue::String(luau.create_string(canonical_path)?))
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

fn fs_path_cwd(luau: &Lua, _value: LuaValue) -> LuaValueResult {
    let function_name = "fs.path.cwd()";
    match std::env::current_dir() {
        Ok(cwd) => {
            if let Some(cwd) = cwd.to_str() {
                Ok(LuaValue::String(luau.create_string(cwd)?))
            } else {
                wrap_err!("{}: cwd is not valid utf-8", function_name)
            }
        },
        Err(err) => {
            wrap_err!("{}: unable to get cwd: {}", function_name, err)
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
        .with_function("exists", super::fs_exists)?
        .with_function("normalize", fs_path_normalize)?
        .with_function("canonicalize", fs_path_canonicalize)?
        .with_function("absolutize", fs_path_absolutize)?
        .with_function("parent", fs_path_parent)?
        .with_function("child", fs_path_child)?
        .with_function("home", fs_path_home)?
        .with_function("cwd", fs_path_cwd)?
        .build_readonly()
}