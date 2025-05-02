use mlua::prelude::*;
use std::{fs, io, path::PathBuf};
use crate::{colors, ok_table, wrap_err, LuaValueResult, TableBuilder};
use super::{entry::{self, wrap_io_read_errors}, validate_path, validate_path_without_checking_fs};

fn fr_exists(_luau: &Lua, multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "FindResult:exists()";
    let search_path = get_search_path(multivalue, function_name)?;
    match fs::exists(&search_path) {
        Ok(exists) => {
            Ok(LuaValue::Boolean(exists))
        },
        Err(err) => {
            wrap_io_read_errors(err, function_name, &search_path)
        }
    }
}

/// creates a FileEntry from FindResult.path if it's a file
/// if fs.find/DirectoryEntry:find was called with { error_if_permission_denied = false } then
/// the user has explicitly intended to check if permission should be denied or not, so
/// if permission was denied, this method will return nil instead of erroring
fn fr_try_file(luau: &Lua, multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "FindResult:try_file()";
    let (search_path, find_result) = get_search_path_and_find_result(multivalue, function_name)?;
    let search_pathbuf = PathBuf::from(&search_path);
    if search_pathbuf.is_file() {
        let file_entry = entry::create(luau, &search_path, function_name)?;
        find_result.raw_set("ok", true)?;
        find_result.raw_set("type", "File")?;
        Ok(file_entry)
    } else {
        find_result.raw_set("ok", false)?;
        find_result.raw_set("type", "NotFound")?;
        Ok(LuaNil)
    }
}

fn fr_try_dir(luau: &Lua, multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "FindResult:try_dir()";
    let (search_path, find_result) = get_search_path_and_find_result(multivalue, function_name)?;
    let search_pathbuf = PathBuf::from(&search_path);
    if search_pathbuf.is_dir() {
        let directory_entry = entry::create(luau, &search_path, function_name)?;
        find_result.raw_set("ok", true)?;
        find_result.raw_set("type", "Directory")?;
        Ok(directory_entry)
    } else {
        find_result.raw_set("ok", false)?;
        find_result.raw_set("type", "NotFound")?;
        Ok(LuaNil)
    }
}

fn fr_unwrap_file(luau: &Lua, multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "FindResult:unwrap_file()";
    let search_path = get_search_path(multivalue, function_name)?;
    let metadata = match fs::metadata(&search_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            match err.kind() {
                io::ErrorKind::NotFound => {
                    return wrap_err!("Attempt to {} a nonexistent file: '{}'", function_name, search_path);
                },
                io::ErrorKind::PermissionDenied => {
                    return wrap_err!("{}: attempt to unwrap file at '{}' but permission denied", function_name, search_path);
                },
                _other => {
                    return wrap_err!("{}: attempt to unwrap file at '{}' but encountered unexpected err: {}", function_name, search_path, err);
                }
            }
        }
    };
    if metadata.is_file() {
        let file_entry = entry::create(luau, &search_path, function_name)?;
        Ok(file_entry)
    } else if metadata.is_dir() {
        wrap_err!("{}: path at '{}' is actually a directory!", function_name, search_path)
    } else {
        wrap_err!("{}: path at '{}' is actually a symlink!", function_name, search_path)
    }
}

fn fr_unwrap_dir(luau: &Lua, multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "FindResult:unwrap_dir()";
    let search_path = get_search_path(multivalue, function_name)?;
    let metadata = match fs::metadata(&search_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            match err.kind() {
                io::ErrorKind::NotFound => {
                    return wrap_err!("Attempt to {} a nonexistent directory: '{}'", function_name, search_path);
                },
                io::ErrorKind::PermissionDenied => {
                    return wrap_err!("{}: attempt to unwrap directory at '{}' but permission denied", function_name, search_path);
                },
                _other => {
                    return wrap_err!("{}: attempt to unwrap directory at '{}' but encountered unexpected err: {}", function_name, search_path, err);
                }
            }
        }
    };
    if metadata.is_dir() {
        let directory_entry = entry::create(luau, &search_path, function_name)?;
        Ok(directory_entry)
    } else if metadata.is_file() {
        wrap_err!("{}: path at '{}' is actually a file!", function_name, search_path)
    } else {
        wrap_err!("{}: path at '{}' is actually a symlink!", function_name, search_path)
    }
}

/// find(path: string, options: { follow_symlinks: boolean?, error_if_permission_denied: boolean? }): FindResult
/// both follow_symlinks and error_if_permission_denied are true by default
pub fn find(luau: &Lua, mut multivalue: LuaMultiValue, function_name: &str) -> LuaValueResult {
     // pop this first; we check it later after determining whether we're erroring if permission denied or not
    let search_path = multivalue.pop_front();

    let find_options = match multivalue.pop_front() {
        Some(LuaValue::Table(options)) => Some(options),
        Some(LuaNil) | None => None,
        Some(other) => {
            return wrap_err!("{} expected options to be an optional table of {{ follow_symlinks: boolean? (default true), error_if_permission_denied: boolean? (default true)}}, got: {:?}", function_name, other);
        }
    };

    let follow_symlinks = if let Some(ref options) = find_options {
        match options.raw_get("follow_symlinks")? {
            LuaValue::Boolean(b) => b,
            LuaNil => true,
            other => {
                return wrap_err!("{} expected options.follow_symlinks to be a boolean (default true) or nil, got: {:?}", function_name, other);
            }
        }
    } else {
        true
    };
    let error_if_permission_denied = if let Some(options) = find_options {
        match options.raw_get("error_if_permission_denied")? {
            LuaValue::Boolean(b) => b,
            LuaNil => true,
            other => {
                return wrap_err!("{} expected options.error_if_permission_denied to be a boolean (default true) or nil, got: {:?}", function_name, other);
            }
        }
    } else {
        true
    };

    let search_path = match search_path {
        Some(LuaValue::String(path)) => {
            if error_if_permission_denied {
                validate_path(&path, function_name)?
            } else {
                validate_path_without_checking_fs(&path, function_name)?
            }
        },
        Some(other) => {
            return wrap_err!("{} expected path to be a string, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected path to be a string, got nothing", function_name);
        }
    };

    let mut permission_denied = false;

    let metadata = {
        match if follow_symlinks { 
            fs::metadata(&search_path) 
        } else { 
            fs::symlink_metadata(&search_path) 
        } {
            Ok(metadata) => Some(metadata),
            Err(err) => {
                match err.kind() {
                    io::ErrorKind::NotFound => None,
                    io::ErrorKind::PermissionDenied => {
                        if error_if_permission_denied {
                            return wrap_err!("{}: permission denied at path '{}'", function_name, &search_path);
                        } else {
                            permission_denied = true;
                            None
                        }
                    },
                    _ => {
                        return wrap_err!("{}: unable to get metadata due to unexpected error: {}", function_name, err);
                    }
                }
            }
        }
    };

    let result = TableBuilder::create(luau)?
        .with_value("path", search_path.clone())?
        .with_function("exists", fr_exists)?
        .with_function("try_file", fr_try_file)?
        .with_function("try_dir", fr_try_dir)?
        .with_function("unwrap_file", fr_unwrap_file)?
        .with_function("unwrap_dir", fr_unwrap_dir)?
        .build()?;

    if permission_denied {
        result.raw_set("ok", false)?;
        result.raw_set("type", "PermissionDenied")?;
    } else if let Some(metadata) = metadata {
        let result_type = if metadata.is_file() {
            "File"
        } else if metadata.is_dir() {
            "Directory"
        } else if metadata.is_symlink() {
            "Symlink"
        } else {
            unreachable!("I'm not a file, directory, nor symlink; what am i?")
        };
        result.raw_set("ok", true)?;
        result.raw_set("type", result_type)?;
    } else {
        result.raw_set("ok", false)?;
        result.raw_set("type", "NotFound")?;
    }
    ok_table(Ok(result))
}

fn get_search_path(mut multivalue: LuaMultiValue, function_name: &str) -> LuaResult<String> {
    match multivalue.pop_front() {
        Some(LuaValue::Table(find_result)) => {
            if let LuaValue::String(search_path) = find_result.raw_get("path")? {
                validate_path_without_checking_fs(&search_path, function_name)
            } else {
                wrap_err!("{} expected FindResult.path to be a string; why did you modify it??")
            }
        },
        Some(other) => {
            wrap_err!("{} expected self to be a FindResult (table), got: {:?}", function_name, other)
        },
        None => {
            wrap_err!("{} incorrectly called without self (expected methodcall syntax)", function_name)
        }
    }
}

fn get_search_path_and_find_result(mut multivalue: LuaMultiValue, function_name: &str) -> LuaResult<(String, LuaTable)> {
    match multivalue.pop_front() {
        Some(LuaValue::Table(find_result)) => {
            if let LuaValue::String(search_path) = find_result.raw_get("path")? {
                match validate_path(&search_path, function_name) {
                    Ok(search_path) => {
                        Ok((search_path, find_result))
                    },
                    Err(err) => { // ? operator can't convert the return error so we have to do this hack
                        wrap_err!("{}", err)
                    }
                }
            } else {
                wrap_err!("{} expected FindResult.path to be a string; why did you modify it??")
            }
        },
        Some(other) => {
            wrap_err!("{} expected self to be a FindResult (table), got: {:?}", function_name, other)
        },
        None => {
            wrap_err!("{} incorrectly called without self (expected methodcall syntax)", function_name)
        }
    }
}