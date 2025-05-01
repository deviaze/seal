use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use mlua::prelude::*;
use crate::{LuaValueResult, LuaEmptyResult, wrap_err, colors, table_helpers::TableBuilder, std_fs};
use crate::{ok_string, ok_table};
use crate::std_fs::entry::{self, wrap_io_read_errors, get_path_from_entry};
use super::pathlib::{normalize_path, path_join};
use super::validate_path;

pub fn listdir(luau: &Lua, dir_path: String, mut multivalue: LuaMultiValue, function_name_and_args: &str) -> LuaValueResult {
    let recursive = match multivalue.pop_front() {
        Some(LuaValue::Boolean(recursive)) => recursive,
        Some(LuaNil) | None => false,
        Some(other) => {
            return wrap_err!("{} expected recursive to be a boolean or nil, got: {:?}", function_name_and_args, other);
        }
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
                let list_path = normalize_path(&list_path);
                entries_list.push(list_path)?;
            }
        } else {
            for entry in fs::read_dir(&dir_path)? {
                let entry = entry?;
                if let Some(entry_path) = entry.path().to_str() {
                    let entry_path = normalize_path(entry_path);
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

fn dir_entries(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let entry_path = match multivalue.pop_front() {
        Some(entry) => get_path_from_entry(&entry, "DirectoryEntry:entries()")?,
        None => {
            return wrap_err!("DirectoryEntry:entries() expected to be called with self");
        }
    };
    std_fs::entries(luau, entry_path.into_lua(luau)?, "DirectoryEntry:entries()")
}

fn dir_find(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "DirectoryEntry:find(path: string, follow_symlinks: boolean?)";
    let entry = match multivalue.pop_front() {
        Some(LuaValue::Table(entry)) => entry,
        Some(other) => {
            return wrap_err!("{} expected self to be a DirectoryEntry, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected to be called with self, did you accidentally use a '.' instead of ':'?", function_name);
        }
    };
    let entry_path = match entry.raw_get("path")? {
        LuaValue::String(path) => {
            validate_path(&path, function_name)?
        },
        other => {
            return wrap_err!("{}: DirectoryEntry is *supposed* to have a 'self' field, got: {:?}; did you remove or modify it?", function_name, other);
        }
    };
    let search_name = match multivalue.pop_front() {
        Some(LuaValue::String(name)) => name.to_str()?.to_string(),
        Some(other) => {
            return wrap_err!("{} expected name to be string, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected name to be string, got nothing", function_name);
        }
    };
    let entry_path = PathBuf::from(&entry_path);
    let search_path = Path::join(&entry_path, search_name);
    let search_path = search_path.to_str();
    match search_path {
        Some(search_path) => {
            multivalue.push_front(search_path.to_string().into_lua(luau)?);
            super::find::find(luau, multivalue, function_name)
        },
        None => {
            wrap_err!("{}: provided path can't be converted to valid utf-8", function_name)
        }
    }
}

fn dir_expect_file(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "DirectoryEntry:expect_file(name: string)";
    let entry = match multivalue.pop_front() {
        Some(LuaValue::Table(entry)) => entry,
        Some(other) => {
            return wrap_err!("{} expected self to be a DirectoryEntry table, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected to be called with self, did you forget to use methodcall (:) syntax?", function_name);
        }
    };
    let entry_path = get_path_from_entry(&LuaValue::Table(entry), function_name)?;
    let name = match multivalue.pop_front() {
        Some(LuaValue::String(name)) => name.to_string_lossy(),
        Some(other) => {
            return wrap_err!("{} expected name to be a string, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected name, got nothing", function_name);
        }
    };
    let mut pathbuf = PathBuf::from(&entry_path);
    pathbuf.push(&name);
    let path = pathbuf.to_string_lossy().to_string();
    match super::file_entry::create(luau, &path) {
        Ok(file_entry) => Ok(LuaValue::Table(file_entry)),
        Err(_err) => {
            wrap_err!("{}: expected file doesn't exist: '{}'", function_name, pathbuf.display())
        }
    }
}

fn dir_expect_dir(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "DirectoryEntry:expect_dir(name: string)";
    let entry = match multivalue.pop_front() {
        Some(LuaValue::Table(entry)) => entry,
        Some(other) => {
            return wrap_err!("{} expected self to be a DirectoryEntry table, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected to be called with self, did you forget to use methodcall (:) syntax?", function_name);
        }
    };
    let entry_path = get_path_from_entry(&LuaValue::Table(entry), function_name)?;
    let name = match multivalue.pop_front() {
        Some(LuaValue::String(name)) => name.to_string_lossy(),
        Some(other) => {
            return wrap_err!("{} expected name to be a string, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected name, got nothing", function_name);
        }
    };
    let mut pathbuf = PathBuf::from(&entry_path);
    pathbuf.push(&name);
    let path = pathbuf.to_string_lossy().to_string();
    match super::directory_entry::create(luau, &path) {
        Ok(file_entry) => Ok(LuaValue::Table(file_entry)),
        Err(_err) => {
            wrap_err!("{}: expected directory doesn't exist: '{}'", function_name, pathbuf.display())
        }
    }
}

fn dir_add_file(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "DirectoryEntry:add_file(name: string, content: string)";
    let entry = match multivalue.pop_front() {
        Some(entry) => entry,
        None => {
            return wrap_err!("{} expected to be called with self, did you accidentally use a '.' instead of ':'?", function_name);
        }
    };
    let entry_path = get_path_from_entry(&entry, function_name)?;
    let name = match multivalue.pop_front() {
        Some(LuaValue::String(name)) => name.to_str()?.to_string(),
        Some(other) => {
            return wrap_err!("{} expected name to be a string, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected name, got nothing", function_name);
        }
    };
    let content = match multivalue.pop_front() {
        Some(LuaValue::String(content)) => content.as_bytes().to_vec(),
        Some(LuaValue::Buffer(buffy)) => buffy.to_vec(),
        Some(other) => {
            return wrap_err!("{} expected content to be string, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected content, got nothing", function_name);
        }
    };
    let mut file_path = PathBuf::from(&entry_path);
    file_path.push(&name);
    match fs::write(&file_path, content) {
        Ok(_) => {
            Ok(entry) // return the DirectoryEntry for chaining
        },
        Err(err) => {
            wrap_io_read_errors(err, function_name, &file_path)
        }
    }
}

fn dir_add_tree(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "DirectoryEntry:add_tree(name: string, tree: TreeBuilder | DirectoryTree)";
    let entry = match multivalue.pop_front() {
        Some(entry) => entry,
        None => {
            return wrap_err!("{} expected to be called with self, did you accidentally use a '.' instead of ':'?", function_name);
        }
    };
    let entry_path = get_path_from_entry(&entry, function_name)?;
    let name = match multivalue.pop_front() {
        Some(LuaValue::String(name)) => name.to_str()?.to_string(),
        Some(other) => {
            return wrap_err!("{} expected name to be a string, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected name, got nothing", function_name);
        }
    };
    let tree = match multivalue.pop_front() {
        Some(LuaValue::Table(tree)) => tree,
        Some(other) => {
            return wrap_err!("{} expected tree to be a TreeBuilder or DirectoryTree, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected tree, got nothing", function_name);
        }
    };
    let mut tree_path = PathBuf::from(&entry_path);
    tree_path.push(&name);
    let path = tree_path.to_string_lossy().to_string();
    std_fs::writetree(luau, path, tree, function_name)?;
    Ok(entry)
}

/// DirectoryEntry:join(paths: ...string)
/// convenience wrapper for path.join around a DirectoryEntry's path
fn dir_join(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "DirectoryEntry:join(paths: ...string)";
    let entry_path = match multivalue.pop_front() {
        Some(entry) => get_path_from_entry(&entry, function_name)?,
        None => {
            return wrap_err!("{} expected to be called with self", function_name);
        }
    };
    let mut paths_to_join = VecDeque::new();
    paths_to_join.push_back(entry_path);

    let mut current_index = 0;
    while let Some(value) = multivalue.pop_front() {
        current_index += 1;
        let string_to_add = match value {
            LuaValue::String(path) => match path.to_str() {
                Ok(str) => str.to_string(),
                Err(_) => {
                    return wrap_err!("{}: path component at index '{}' is not valid utf-8 encoded", function_name, current_index);
                }
            }
            other => {
                return wrap_err!("{} expected path component at index '{}' to be a string, got: {:?}", function_name, current_index, other);
            }
        };
        paths_to_join.push_back(string_to_add);
    }

    ok_string(path_join(paths_to_join), luau)
}

pub fn create(luau: &Lua, path: &str) -> LuaResult<LuaTable> {
    let original_path = path;
    let path = PathBuf::from(path);
    if !path.exists() || !path.is_dir() {
        return wrap_err!("Directory not found: '{}'", path.display());
    }
    let base_name = match path.file_name() {
        Some(name) => {
            match name.to_str() {
                Some(name) => name,
                None => {
                    return wrap_err!("unable to create DirectoryEntry; the name of the file at path {} is non-unicode", path.display());
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
        .with_function("join", dir_join)?
        .with_function("entries", dir_entries)?
        .with_function("add_file", dir_add_file)?
        .with_function("add_tree", dir_add_tree)?
        .with_function("find", dir_find)?
        .with_function("expect_file", dir_expect_file)?
        .with_function("expect_dir", dir_expect_dir)?
        .with_function("metadata", entry::metadata)?
        .with_function("copy_to", entry::copy_to)?
        .with_function("move_to", entry::move_to)?
        .with_function("rename", entry::rename)?
        .with_function("remove", entry::remove)?
        .build()
}