use mluau::prelude::*;
use std::collections::VecDeque;
use std::io;
use std::fs;
use std::path::{self, Path};
use crate::globals;
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
        "./" | "../" | "." | "" | "/" | "~/" => "/", // unix style '/' (default) that works on windows, linux, unix, macos, etc.
        ".\\" | "..\\" | "~\\" => "\\", // windows style '\' for windows absolute paths
        // stupid windows absolute path edge cases
        component if component.ends_with(':') => "\\", // handle drive letters like "C:"
        component if component.starts_with(r"\\") => "\\", // absolute paths starting with backslash (\\wsl\\)
        component if component.contains(':') => "\\",     // paths with drive letters (e.g., "C:\")
        _ => "/", // probably a path.join("dogs", "cats") partial path, default to /
    };

    let mut result = String::new();

    // handle absolute paths/roots
    match first_component.as_str() {
        "/" => {},
        r"\\" => {
            // push first \ of \\ unc path; for loop pushes the next \ so unc path ends up starting with \\
            result.push('\\');
        },
        first if first.starts_with("/") || first.starts_with(r"\\") => {
            // prevent stripping leading unix root / or windows unc root \\
            // if component contains either but isn't exactly either
            result.push_str(first.trim_end_matches(['/', '\\']));
        },
        other => {
            result.push_str(trim_path(other));
        }
    };

    for component in components {
        let trimmed_component = trim_path(&component);
        if !trimmed_component.is_empty() {
            result.push_str(path_sep);
            result.push_str(trimmed_component);
        }
    }

    result
}

pub fn normalize_path<P: AsRef<str>>(path: P) -> String {
    let path = path.as_ref();

    let slash = "/";
    let backslash = r"\";

    // we default to / separators except when dealing with windows absolute paths
    // or the user explicitly passed a path starting with .\ or ..\ as the first param
    let mut standardized_separator = slash;

    let mut new_components = Vec::<&str>::with_capacity(path.len());
    if path.len() < 3 {
        return path.to_string(); // give back any baby paths so we don't panic on the following range index
    }

    let first_two = &path[..2];
    let third = &path[2..=2];

    let mut is_windows_drive_path = false;

    // check for the first most common prefixes
    let mut root = match first_two {
        // relative paths on unix-like and our default for windows relative paths
        "./" => "./",
        // relative windows paths
        r".\" => {
            standardized_separator = backslash; // preserve user intent
            r".\"
        },
        // parent paths
        ".." if third == slash => "../", // parent path
        ".." if third == backslash => { // windows parent path
            standardized_separator = backslash;
            r"..\"
        },
        // home dirs /home/<user> on linux or C:\Users\<User> on Windows
        "~/" => "~/",
        r"~\" => {
            standardized_separator = backslash;
            r"~\"
        },
        // windows unc paths
        r"\\" => {
            standardized_separator = backslash;
            r"\\"
        },
        // unix/linux/macos absolute paths
        other if other.starts_with("/") => {
            other
        },
        // windows drive letter absolute path like C:\Users\deviaze\meow
        other if other.chars().nth(1) == Some(':') => {
            standardized_separator = backslash;
            is_windows_drive_path = true;
            other
        },
        other => other,
    }.to_owned();

    // fix C: to be C:\ for path joining
    if is_windows_drive_path { 
        root.push('\\');
    }

    // slice path by the remaining
    let path = &path[2..];
    let mut old_components = path
        .split(['/', '\\'])
        .filter(|s| !s.is_empty())
        .collect::<VecDeque<&str>>();

    while let Some(component) = old_components.pop_front() {
        match component {
            "." => continue, // skip redundant . in middle of ./animals/cats/./meow.jpg
            ".." => {
                if let Some(last) = new_components.last() 
                    && *last != ".." // redundant .. we can normalize by popping last parent
                {
                    let _ = new_components.pop();
                } else { // if the last is .. then we have a ../../../situation going on we don't want to pop
                    new_components.push("..");
                }
            },
            other => {
                new_components.push(other);
            }
        }
    }

    root + &new_components.join(standardized_separator)
}

pub fn canonicalize_path(path: &str) -> LuaResult<String> {
    match fs::canonicalize(path) {
        Ok(canonical_path) => {
            #[allow(unused_mut, reason = "needs to be mut on windows")]
            let mut canonical_path = canonical_path.to_string_lossy().to_string();
            #[cfg(windows)]
            {
                // very cool unc paths windows
                canonical_path = canonical_path.replace(r"\\?\", "");
            }
            // Ok(LuaValue::String(luau.create_string(canonical_path)?))
            Ok(canonical_path)
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
    ok_string(result, luau)
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

    ok_string(canonicalize_path(&path)?, luau)
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

    ok_string(current_path.to_string_lossy().to_string(), luau)
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
            ok_string(name, luau)
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
                ok_string(cwd, luau)
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
    if let Some(home_dir) = std::env::home_dir() {
        let home_dir = home_dir.to_string_lossy().to_string();
        ok_string(home_dir, luau)
    } else {
        Ok(LuaNil)
    }
}

fn fs_path_project(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "fs.path.project(n: number?, script_path: string?)";
    let projects_up = match multivalue.pop_front() {
        Some(LuaValue::Integer(i)) => int_to_usize(i, function_name, "n")?,
        Some(LuaValue::Number(f))=> float_to_usize(f, function_name, "n")?,
        Some(LuaNil) | None => 1,
        Some(other) => {
            return wrap_err!("{} expected n, the number of projects up (default 1) to be a number or nil/unspecified, got: {:?}", function_name, other);
        }
    };
    let script_path = match multivalue.pop_front() {
        Some(LuaValue::String(entry)) => entry.to_string_lossy(),
        Some(LuaNil) | None => globals::get_debug_name(luau)?,
        Some(other) => {
            return wrap_err!("{} expected script_path to be a string or nil, got: {:?}", function_name, other);
        }
    };
    match globals::find_project(&script_path, projects_up) {
        Some(project) => ok_string(project.to_string_lossy().to_string(), luau),
        None => Ok(LuaNil)
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
        .with_function("project", fs_path_project)?
        .build_readonly()
}