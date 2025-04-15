use std::fs;
use std::io;
use std::path::PathBuf;
use mlua::prelude::*;
use crate::{wrap_err, colors, LuaValueResult, TableBuilder};
use crate::require::ok_table;
use super::entry::wrap_io_read_errors;
use super::{directory_entry, entry, validate_path};

fn fs_dir_from(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let path = match value {
        LuaValue::String(path) => path.to_string_lossy(),
        other => {
            return wrap_err!("fs.dir.from(path) expected path to be a string, got: {:#?}", other);
        }
    };
    ok_table(directory_entry::create(luau, &path))
}

fn fs_dir_build(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let dir_name = match multivalue.pop_front() {
        Some(LuaValue::String(name)) => name,
        Some(other) => {
            return wrap_err!("fs.dir.build(name: string, children: DirectoryTree) expected name to be a string, got: {:?}", other);
        },
        None => {
            return wrap_err!("fs.dir.build(name: string, children: DirectoryTree) expected name, got nothing");
        }
    };
    let children = match multivalue.pop_front() {
        Some(LuaValue::Table(children)) => children,
        Some(other) => {
            return wrap_err!("fs.dir.build(name: string, children: DirectoryTree) expected children to be a DirectoryTree table (an array-like-table of tables from fs.file.build or fs.dir.build), got: {:?}", other);
        },
        None => {
            return wrap_err!("fs.dir.build(name: string, children: DirectoryTree) expected children, got nothing");
        }
    };
    ok_table(TableBuilder::create(luau)?
        .with_value("type", "Directory")?
        .with_value("name", dir_name)?
        .with_value("children", children)?
        .build()
    )
}

fn fs_dir_call(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "fs.dir:__call(path: string)";
    let Some(LuaValue::Table(_filelib)) = multivalue.pop_front() else {
        return wrap_err!("{}: somehow called without self (or where self isn't a table)? this is impossible", function_name);
    };
    let dir_path = match multivalue.pop_front() {
        Some(LuaValue::String(path)) => {
            validate_path(&path, function_name)?
        },
        Some(other) => {
            return wrap_err!("{} expected path to be a string, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected path to be a string, got nothing", function_name);
        }
    };

    let metadata = match fs::metadata(&dir_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            match err.kind() {
                io::ErrorKind::NotFound => {
                    return Ok(LuaNil)
                },
                io::ErrorKind::PermissionDenied => {
                    return wrap_err!("{}: permission denied at '{}'", function_name, dir_path);
                },
                _other => {
                    return wrap_err!("{}: unexpected error at '{}', err: {}", function_name, dir_path, err);
                }
            }
        }
    };
    
    if metadata.is_dir() {
        entry::create(luau, &dir_path, function_name)
    } else {
        Ok(LuaNil)
    }
}

fn fs_dir_create(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "fs.dir.create(path: string)";
    let path = match value {
        LuaValue::String(path) => {
            validate_path(&path, function_name)?
        },
        other => {
            return wrap_err!("{} expected path to be a string, got: {:?}", function_name, other);
        }
    };
    match fs::create_dir(&path) {
        Ok(_) => {
            entry::create(luau, &path, function_name)
        },
        Err(err) => {
            wrap_io_read_errors(err, function_name, &path)
        }
    }
}

/// fs.dir.ensure(path: string, create_missing: boolean?): DirectoryEntry
/// ensures a directory exists at the requested path by making it (AlreadyExists is ok)
/// and creating a DirectoryEntry for that path.
/// Basically similar to fs.makedir(path, { error_if_exists = false }) + fs.dir.from(path)
/// Errors if PermissionDenied or whatever's at that path is actually a File
fn fs_dir_ensure(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "fs.dir.ensure(path: string, create_missing: boolean?)";
    let requested_path = match multivalue.pop_front() {
        Some(LuaValue::String(path)) => {
            validate_path(&path, function_name)?
        },
        Some(LuaNil) => {
            return wrap_err!("{} expected path to be a string, got nil", function_name);
        },
        Some(other) => {
            return wrap_err!("{} expected path to be a string, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected path but was incorrectly called with no arguments", function_name);
        }
    };
    let create_missing = match multivalue.pop_front() {
        Some(LuaValue::Boolean(b)) => b,
        Some(LuaNil) | None => false,
        Some(other) => {
            return wrap_err!("{} expected create_missing to be an optional boolean (default false), got: {:?}", function_name, other);
        }
    };
    let requested_pathbuf = PathBuf::from(&requested_path);
    let already_exists = match if create_missing {
        fs::create_dir_all(&requested_pathbuf)
    } else {
        fs::create_dir(&requested_pathbuf)
    } {
        Ok(_) => false,
        Err(err) => {
            match err.kind() {
                io::ErrorKind::AlreadyExists => true,
                io::ErrorKind::NotFound => {
                    return wrap_err!("{} cannot create directory; path to '{}' doesn't exist; pass 'true' as the second argument to fs.dir.ensure to create the missing directories", function_name, requested_path);
                }
                io::ErrorKind::PermissionDenied => {
                    return wrap_err!("{} cannot ensure directory at '{}' exists (Permission Denied)", function_name, requested_path);
                },
                _other => {
                    return wrap_err!("{}: unexpected error when trying to create directory at '{}': {}", function_name, requested_path, err);
                }
            }
        }
    };
    if already_exists && requested_pathbuf.is_file() {
        return wrap_err!("{} cannot ensure directory at '{}' exists: it's actually a file!", function_name, requested_path);
    }
    entry::create(luau, &requested_path, function_name)
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("from", fs_dir_from)?
        .with_function("build", fs_dir_build)?
        .with_function("create", fs_dir_create)?
        .with_function("ensure", fs_dir_ensure)?
        .with_metatable(TableBuilder::create(luau)?
            .with_function("__call", fs_dir_call)?
            .build_readonly()?
        )?
        .build_readonly()
}