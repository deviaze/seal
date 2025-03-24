use std::fs;
use std::path::PathBuf;
use mlua::prelude::*;
use crate::{LuaValueResult, LuaEmptyResult, wrap_err, colors, table_helpers::TableBuilder, std_fs};
use crate::require::ok_table;
use crate::std_fs::entry::{self, wrap_io_read_errors, wrap_io_read_errors_empty, get_path_from_entry};

pub fn listdir(luau: &Lua, dir_path: String, mut multivalue: LuaMultiValue, function_name_and_args: &str) -> LuaValueResult {
    let recursive = match multivalue.pop_front() {
        Some(LuaValue::Boolean(recursive)) => recursive,
        Some(LuaNil) => false,
        Some(other) => {
            return wrap_err!("{} expected recursive to be a boolean (default false), got: {:#?}", function_name_and_args, other);
        }
        None => false,
    };

    let metadata = match fs::metadata(&dir_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return wrap_io_read_errors(err, function_name_and_args, &dir_path);
        }
    };
    if metadata.is_dir() {
        let entries_list = luau.create_table()?;
        if recursive {
            let mut list_vec = Vec::new();
            match list_dir_recursive(&dir_path, &mut list_vec) {
                Ok(()) => {},
                Err(err) => {
                    return wrap_err!("{}: unable to recursively iterate over path: {}", function_name_and_args, err);
                }
            };
            let list_vec = list_vec; // make immutable again
            for list_path in list_vec {
                entries_list.push(list_path)?;
            }
        } else {
            for entry in fs::read_dir(&dir_path)? {
                let entry = entry?;
                if let Some(entry_path) = entry.path().to_str() {
                    entries_list.push(entry_path)?;
                }
            };
        }
        ok_table(Ok(entries_list))
    } else {
        wrap_err!("{}: expected path at '{}' to be a directory, but found a file", function_name_and_args, &dir_path)
    }
}

// modifies the passed Vec<String> in place
fn list_dir_recursive(path: &str, list: &mut Vec<String>) -> LuaEmptyResult {
    for entry in (fs::read_dir(path)?).flatten() {
        let current_path = entry.path();
        if current_path.is_dir() {
            if let Some(current_path) = current_path.to_str() {
                list_dir_recursive(current_path, list)?;
            } else {
                continue; // path contains invalid utf8 but we're ignoring it
            }
        } else if let Some(path_string) = current_path.to_str() {
            list.push(path_string.to_string())
        }
    }
    Ok(())
}

fn dir_list(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let entry_path = match multivalue.pop_front() {
        Some(entry) => get_path_from_entry(&entry, "DirectoryEntry:list()")?,
        None => {
            return wrap_err!("DirectoryEntry:list() expected to be called with self");
        }
    };
    listdir(luau, entry_path, multivalue, "DirectoryEntry:list(recursive: boolean?)")
}

pub fn create(luau: &Lua, path: &str) -> LuaResult<LuaTable> {
    let original_path = path;
    let path = PathBuf::from(path);
    if !path.exists() {
        return wrap_err!("Directory not found: '{}'", path.display());
    }
    let base_name = match path.file_name() {
        Some(name) => {
            match name.to_str() {
                Some(name) => name,
                None => {
                    return wrap_err!("unable to create FileEntry; the name of the file at path {} is non-unicode", path.display());
                }
            }
        },
        None => "",
    };
    TableBuilder::create(luau)?
        .with_value("name", base_name)?
        .with_value("path", original_path)?
        .with_value("type", "Directory")?
        .with_function("list", dir_list)?
        .build()
}