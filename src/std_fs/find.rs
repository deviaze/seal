use mlua::prelude::*;
use std::{fs, io};
use crate::{colors, require::ok_table, wrap_err, LuaValueResult, TableBuilder};
use super::{entry::{self, wrap_io_read_errors}, validate_path};

pub fn find(luau: &Lua, mut multivalue: LuaMultiValue, function_name: &str) -> LuaValueResult {
    let search_path = match multivalue.pop_front() {
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
    let follow_symlinks = match multivalue.pop_front() {
        Some(LuaValue::Boolean(follow)) => follow,
        Some(LuaNil) => true,
        Some(other) => {
            return wrap_err!("{} expected follow_symlinks to be a boolean or nil (or left unspecified), got: {:?}", function_name, other);
        },
        None => true,
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
                        // return wrap_err!("{}: Permission denied at path '{}'", function_name, &search_path);
                        permission_denied = true;
                        None
                    },
                    _ => {
                        return wrap_err!("{}: unable to get metadata due to error: {}", function_name, err);
                    }
                }
            }
        }
    };

    let search_path_to_check_existence = search_path.clone();
    
    let check_exists = move | _luau: &Lua, _value: LuaValue | -> LuaValueResult {
        match fs::exists(&search_path_to_check_existence) {
            Ok(bool) => Ok(LuaValue::Boolean(bool)),
            Err(err) => wrap_io_read_errors(err, "FindResult:exists()", &search_path_to_check_existence)
        }
    };

    if permission_denied {
        ok_table(TableBuilder::create(luau)?
            .with_value("ok", false)?
            .with_value("err", "PermissionDenied")?
            .with_function("exists", check_exists)?
            .with_function("retry_file", {
                | _luau: &Lua, multivalue: LuaMultiValue | -> LuaValueResult {
                    let search_path = get_search_path(multivalue, "retry_file")?;
                    wrap_err!("FindResult:retry_file(): Permission denied at '{}'", search_path)
                }
            })?
            .with_function("retry_dir", {
                | _luau: &Lua, multivalue: LuaMultiValue | -> LuaValueResult {
                    let search_path = get_search_path(multivalue, "retry_dir")?;
                    wrap_err!("FindResult:retry_dir(): Permission denied at '{}'", search_path)
                }
            })?
            .with_function("unwrap_file", {
                | _luau: &Lua, multivalue: LuaMultiValue | -> LuaValueResult {
                    let search_path = get_search_path(multivalue, "unwrap_file")?;
                    wrap_err!("FindResult:unwrap_file(): Permission denied at '{}'", search_path)
                }
            })?
            .with_function("unwrap_dir", {
                | _luau: &Lua, multivalue: LuaMultiValue | -> LuaValueResult {
                    let search_path = get_search_path(multivalue, "unwrap_dir")?;
                    wrap_err!("FindResult:unwrap_dir(): Permission denied at '{}'", search_path)
                }
            })?
            .build_readonly()
        )
    } else {
        let search_path_clone = search_path.clone();
        let find_result = TableBuilder::create(luau)?
            .with_value("ok", true)?
            .with_value("path", search_path_clone)?
            .with_function("exists", check_exists)?
            .with_function("unwrap_file", {
                | _luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                    match multivalue.pop_front() {
                        Some(LuaValue::Table(find_result)) => {
                            if let Ok(LuaValue::Table(file)) = find_result.raw_get("file") {
                                ok_table(Ok(file))
                            } else {
                                wrap_err!("Attempt to :unwrap_file() a non-file FindResult")
                            }
                        },
                        Some(other) => {
                            wrap_err!("FindResult:unwrap_file(): expected self to be a FindResult, got: {:?}", other)
                        },
                        None => {
                            wrap_err!("FindResult:unwrap_file() incorrectly called without self")
                        }
                    }
                }
            })?
            .with_function("unwrap_dir", {
                | _luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                    match multivalue.pop_front() {
                        Some(LuaValue::Table(find_result)) => {
                            if let Ok(LuaValue::Table(dir)) = find_result.raw_get("dir") {
                                ok_table(Ok(dir))
                            } else {
                                wrap_err!("Attempt to :unwrap_dir() a non-dir FindResult")
                            }
                        },
                        Some(other) => {
                            wrap_err!("FindResult:unwrap_dir(): expected self to be a FindResult, got: {:?}", other)
                        },
                        None => {
                            wrap_err!("FindResult:unwrap_dir() incorrectly called without self")
                        }
                    }
                }
            })?
            .with_function("retry_file", {
                | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                    let find_result = match multivalue.pop_front() {
                        Some(LuaValue::Table(entry)) => entry,
                        Some(other) => {
                            return wrap_err!("FindResult:retry_file() expected self to be a FindResult, got: {:?}", other);
                        },
                        None => {
                            return wrap_err!("FindResult:retry_file() incorrectly called without self");
                        }
                    };
                    let entry_path: String = find_result.raw_get("path")?;
                    let exists = match fs::exists(&entry_path) {
                        Ok(exists) => exists,
                        Err(err) => {
                            return wrap_io_read_errors(err, "FindResult:retry_file()", &entry_path);
                        }
                    };
                    if exists {
                        let metadata = match fs::metadata(&entry_path) {
                            Ok(metadata) => metadata,
                            Err(err) => {
                                return wrap_io_read_errors(err, "FindResult:retry_file()", &entry_path);
                            }
                        };
                        if metadata.is_file() {
                            let new_entry = super::entry::create(luau, &entry_path, "FindResult:retry_file()")?;
                            find_result.raw_set("file", new_entry)?;
                            let new_entry: LuaValue = find_result.raw_get("file")?;
                            Ok(new_entry)
                        } else {
                            Ok(LuaNil)
                        }
                    } else {
                        find_result.raw_set("file", LuaNil)?; // in case retry_file called after file removed
                        Ok(LuaNil)
                    }
                }
            })?
            .with_function("retry_dir", {
                | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                    let find_result = match multivalue.pop_front() {
                        Some(LuaValue::Table(entry)) => entry,
                        Some(other) => {
                            return wrap_err!("FindResult:retry_dir() expected self to be a FindResult, got: {:?}", other);
                        },
                        None => {
                            return wrap_err!("FindResult:retry_dir() incorrectly called without self");
                        }
                    };
                    let entry_path: String = find_result.raw_get("path")?;
                    let exists = match fs::exists(&entry_path) {
                        Ok(exists) => exists,
                        Err(err) => {
                            return wrap_io_read_errors(err, "FindResult:retry_dir()", &entry_path);
                        }
                    };
                    if exists {
                        let metadata = match fs::metadata(&entry_path) {
                            Ok(metadata) => metadata,
                            Err(err) => {
                                return wrap_io_read_errors(err, "FindResult:retry_dir()", &entry_path);
                            }
                        };
                        if metadata.is_dir() {
                            let new_entry = entry::create(luau, &entry_path, "FindResult:retry_dir()")?;
                            find_result.raw_set("dir", new_entry)?;
                            let new_entry: LuaValue = find_result.raw_get("dir")?;
                            Ok(new_entry)
                        } else {
                            Ok(LuaNil)
                        }
                    } else {
                        find_result.raw_set("dir", LuaNil)?; // in case retry_dir called after removed
                        Ok(LuaNil)
                    }
                }
            })?
            .build()?;

        let entry = entry::create(luau, &search_path, function_name)?;
        match metadata {
            Some(metadata) if metadata.is_file() => {
                find_result.raw_set("file", entry)?;
            },
            Some(metadata) if metadata.is_dir() => {
                find_result.raw_set("dir", entry)?;
            },
            Some(_metadata) => {
                todo!("handle symlinks")
            },
            None => {

            }
        }

        ok_table(Ok(find_result))
    }
}

/// helper function for fs_find
fn get_search_path(mut multivalue: LuaMultiValue, function_name: &str) -> LuaResult<String> {
    match multivalue.pop_front() {
        Some(LuaValue::Table(find_result)) => {
            let search_path: LuaString = find_result.raw_get("path")?;
            validate_path(&search_path, function_name)
        },
        Some(other) => {
            wrap_err!("FindResult:{}(): expected self to be a FindResult (table), got: {:?}", function_name, other)
        },
        None => {
            wrap_err!("FindResult{}(): expected self to be a FindResult, got nothing", function_name)
        }
    }
}