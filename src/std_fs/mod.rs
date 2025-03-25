use entry::{wrap_io_read_errors, wrap_io_read_errors_empty};
use mlua::prelude::*;
use regex::SubCaptureMatches;
use std::path::{Path, PathBuf};
use std::{fs, io};
use crate::require::ok_table;
use crate::{table_helpers::TableBuilder, LuaValueResult};
use crate::{std_io_colors as colors, wrap_err, LuaEmptyResult};

pub mod entry;
pub mod pathlib;
pub mod file_entry;
pub mod directory_entry;

pub fn validate_path(path: &LuaString, function_name: &str) -> LuaResult<String> {
    let Ok(path) = path.to_str() else {
        return wrap_err!("{}: provided path '{}' is not properly utf8-encoded", function_name, path.display());
    };
    let path = path.to_string();
    if cfg!(target_os="linux") {
        if !fs::exists(&path)? && fs::exists("/".to_string() + &path)? {
            return wrap_err!("{}: provided path '{}' doesn't exist but an absolute path of it ('/{}') does; did you mean to prepend '/' to it?", function_name, &path, &path);
        } else if !fs::exists(&path)? && path.starts_with("home") { // /home/user/ is ~ on linux
            return wrap_err!("{}: path '{}' looks like an absolute path but doesn't start with '/', did you mean to provide an absolute path?", function_name, &path);
        }
    }
    Ok(path)
}

/// `fs.readfile(path: string): string`
/// 
/// note that we allow reading invalid utf8 files instead of failing (requiring fs.readbytes) 
/// or replacing with utf8 replacement character
/// 
/// this is because Luau allows strings to be of arbitrary encoding unlike Rust, where they have to be utf8 
pub fn fs_readfile(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let file_path = match value {
        LuaValue::String(file_path) => {
            validate_path(&file_path, "fs.readfile(path: string)")?
        },
        other => {
            return wrap_err!("fs.readfile(path: string) expected string, got {:#?}", other);
        }
    };
    let bytes = match fs::read(&file_path) {
        Ok(bytes) => bytes,
        Err(err) => {
            return wrap_io_read_errors(err, "fs.readfile(path: string)", &file_path);
        }
    };
    Ok(LuaValue::String(luau.create_string(bytes)?))
}

/// fs.readbytes(path: string, target_buffer: buffer, buffer_offset: number?, file_offset: number?, count: number)
pub fn fs_readbytes(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let function_name_and_args = "fs.readbytes(path: string, target_buffer: buffer, buffer_offset: number?, file_offset: number?, count: number)";
    let entry_path: String = match multivalue.pop_front() {
        Some(LuaValue::String(file_path)) => {
            validate_path(&file_path, function_name_and_args)?
        },
        Some(other) => 
            return wrap_err!("{} expected path to be a string, got: {:#?}", function_name_and_args, other),
        None => {
            return wrap_err!("{} incorrectly called with zero arguments", function_name_and_args);
        }
    };
    file_entry::read_file_into_buffer(luau, &entry_path, multivalue, function_name_and_args)?;
    Ok(())
}

/// iterate over the lines of a file. you can use this within a for loop
/// or put the function this returns in a local and call it repeatedly ala `local nextline = fs.readlines(filepath); local i, line = nextline()`
fn fs_readlines(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let file_path = match value {
        LuaValue::String(path) => {
            validate_path(&path, "fs.readlines(path: string)")?
        },
        other => {
            return wrap_err!("fs.readlines(path: string): expected a file path, got: {:#?}", other);
        }
    };
    file_entry::readlines(luau, &file_path, "fs.readlines(path: string)")
}

// fs.writefile(path: string, content: string | buffer): ()
pub fn fs_writefile(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let file_path = match multivalue.pop_front() {
        Some(LuaValue::String(path)) => {
            validate_path(&path, "fs.writefile(path: string, content: string | buffer)")?
        },
        Some(other) => {
            return wrap_err!("fs.writefile(path: string, content: string | buffer) expected path to be a string, got: {:#?}", other);
        }
        None => {
            return wrap_err!("fs.writefile(path: string, content: string | buffer) expected path to a be a string, got nothing");
        }
    };
    let content = match multivalue.pop_front() {
        Some(LuaValue::String(content)) => {
            content.as_bytes().to_vec()
        },
        Some(LuaValue::Buffer(content)) => {
            content.to_vec()
        },
        Some(other) => {
            return wrap_err!("fs.writefile(path: string, content: string | buffer) expected content to be a string or buffer, got: {:#?}", other);
        },
        None => {
            return wrap_err!("fs.writefile(path: string, content: string | buffer) expected second argument content to be a string or buffer, got nothing");
        }
    };
    match fs::write(&file_path, &content) {
        Ok(_) => {
            Ok(())
        },
        Err(err) => {
            match err.kind() {
                io::ErrorKind::NotFound => {
                    // if we dont special-case this, it results in an "fs.writefile: File not found {newfilepath}"
                    // error that's like... duh, of course it's not found.. i'm trying to make the file there??
                    // turns out we get NotFounds on fs::write if any of the parent directories don't exist
                    wrap_err!("fs.writefile: path to '{}' doesn't exist, are all directories present and does the path start with '/', './', or '../'?")
                },
                _ => {
                    entry::wrap_io_read_errors_empty(err, "fs.writefile", &file_path)
                }
            }
        }
    }
}

/// fs.removefile(path: string): ()
/// cannot remove directories
pub fn fs_removefile(_luau: &Lua, value: LuaValue) -> LuaEmptyResult {
    let victim_path = match value {
        LuaValue::String(path) => {
            validate_path(&path, "fs.removefile(path: string)")?
        },
        other => {
            return wrap_err!("fs.removefile(path: string) expected path to be a string, got: {:?}", other);
        }
    };
    let metadata = match fs::metadata(&victim_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return wrap_io_read_errors_empty(err, "fs.removefile(path: string)", &victim_path);
        }
    };
    if metadata.is_file() {
        match fs::remove_file(&victim_path) {
            Ok(_) => Ok(()),
            Err(err) => {
                wrap_io_read_errors_empty(err, "fs.removefile(path: string)", &victim_path)
            }
        }
    } else { // it can't be a symlink as fs::metadata follows symlinks
        wrap_err!("fs.removefile(path: string): cannot remove file; path at '{}' is actually a directory!", victim_path)
    }
}

pub fn fs_move(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let from_path = match multivalue.pop_front() {
        Some(LuaValue::String(from)) => {
            validate_path(&from, "fs.move(from: string, to: string)")?
        },
        Some(other) => {
            return wrap_err!("fs.move(from: string, to: string) expected 'from' to be a string, got: {:?}", other);
        },
        None => {
            return wrap_err!("fs.move(from: string, to: string) expected 'from', got nothing");
        }
    };
    let to_path = match multivalue.pop_front() {
        Some(LuaValue::String(to)) => {
            validate_path(&to, "fs.move(from: string, to: string)")?
        },
        Some(other) => {
            return wrap_err!("fs.move(from: string, to: string) expected 'to' to be a string, got: {:?}", other);
        },
        None => {
            return wrap_err!("fs.move(from: string, to: string) expected 'to', got nothing");
        }
    };
    match fs::rename(&from_path, &to_path) {
        Ok(_) => Ok(()),
        Err(err) => {
            wrap_err!("fs.move: unable to move '{}' -> '{}' due to err: {}", from_path, to_path, err)
        }
    }
}

/// fs.readtree(path: string): DirectoryTree
/// not called readdir because it's uglier + we want dir/tree stuff to autocomplete after file
/// so we want fs.readfile to autocomplete first and i'm assuming it's alphabetical
fn _fs_readtree(_luau: &Lua, _value: LuaValue) -> LuaValueResult {
    todo!()
}

/// fs.writetree(path: string, tree: DirectoryTree): ()
fn fs_writetree(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let function_name = "fs.writetree(path: string, tree: DirectoryTree)";
    let path = match multivalue.pop_front() {
        Some(LuaValue::String(path)) => {
            validate_path(&path, function_name)?
        },
        Some(other) => {
            return wrap_err!("{} expected path to be a string, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected path, got nothing", function_name);
        }
    };
    let tree = match multivalue.pop_front() {
        Some(LuaValue::Table(tree)) => {
            match tree.raw_get("type")? {
                LuaValue::String(_) => {
                    return wrap_err!("{} expected tree to be an array-like table which contains entries from fs.dir.build or fs.file.build; did you accidentally pass fs.dir.build to fs.writetree directly?", function_name);
                },
                LuaNil => {},
                _other => {
                    return wrap_err!("{} expected tree to be an array-like table, got an invalid table", function_name);
                }
            };
            match tree.raw_get(1)? {
                LuaValue::Table(_) => tree,
                other => {
                    return wrap_err!("{} expected tree to be an array-like table (that contains entries from fs.file.build or fs.dir.build), got: {:?}", function_name, other);
                }
            }
        },
        Some(LuaNil) | None => {
            return wrap_err!("{} expected tree, got nothing or nil; to create an empty directory, use fs.makedir or pass an empty table as tree", function_name);
        }
        Some(other) => {
            return wrap_err!("{} expected tree to be a DirectoryTree (use fs.dir.build to create DirectoryTrees), got: {:?}", function_name, other);
        },
    };

    match fs::create_dir(&path) {
        Ok(_) => {},
        Err(err) => {
            match err.kind() {
                io::ErrorKind::AlreadyExists => {
                    return wrap_err!("{}: unable to create top-level directory at '{}' because it already exists", function_name, &path);
                },
                _ => {
                    return wrap_err!("{}: unable to create top-level directory at requested path '{}' due to err: {}", function_name, &path, err);
                }
            }
        }
    };

    let path = PathBuf::from(&path);

    write_tree_rec(path, tree, None, function_name)
}

fn write_tree_rec(current_path: PathBuf, tree: LuaTable, depth: Option<i32>, function_name: &str) -> LuaEmptyResult {
    for pair in tree.pairs() {
        let (i, value): (LuaValue, LuaValue) = pair?;
        let LuaValue::Integer(index) = i else {
            return wrap_err!("{} expected tree to be an array-like table, got a non array-like table (non-integer key)", function_name);
        };
        let LuaValue::Table(value) = value else {
            return wrap_err!("{} expected tree to be an array-like table with values being Builders from fs.file.build or fs.dir.build, got a non-table value", function_name);
        };

        let entry_name = match value.raw_get("name")? {
            LuaValue::String(name) => name.to_str()?.to_string(),
            other => {
                return wrap_err!("{}: when evaluating children of path '{}', expected field 'name' of table at index {} to be a string, got: {:?}", function_name, current_path.display(), index, other);
            }
        };

        let build_type = match value.raw_get("type")? {
            LuaValue::String(ty) => {
                match ty.to_str()?.to_string().as_str() {
                    "File" => "File",
                    "Directory" => "Directory",
                    other => {
                        return wrap_err!("{}: when evaluating children of path '{}', expected field 'type' of table at index {} to be either \"File\" or \"Directory\", got: \"{}\"", function_name, current_path.display(), index, other);
                    }
                }
            },
            other => {
                return wrap_err!("{}: when evaluating children of path '{}', expected field 'type' of table at index {} to be a string, got: {:?}", function_name, current_path.display(), index, other);
            }
        };

        let new_entry_pathbuf = Path::join(&current_path, &entry_name);

        if build_type == "File" {
            let content= match value.raw_get("content")? {
                LuaValue::String(content) => {
                    content.as_bytes().to_vec()
                },
                LuaValue::Buffer(buffy) => {
                    buffy.to_vec()
                },
                other => {
                    return wrap_err!("{}: when evaluating content to write to file '{}', expected field 'content' of table to be string (or buffer), got: {:?}", function_name, new_entry_pathbuf.display(), other);
                }
            };

            match fs::write(&new_entry_pathbuf, content) {
                Ok(_) => {},
                Err(err) => {
                    return wrap_err!("{}: error writing to file at '{}': {}", function_name, new_entry_pathbuf.display(), err);
                }
            }
        } else {
            let subtree = match value.raw_get("children")? {
                LuaValue::Table(subtree) => subtree,
                other => {
                    return wrap_err!("{}: when evaluating children of new subtree '{}', expected field 'children' to be a table (array-like, values being returns from fs.dir.build or fs.file.build), got: {:?}", function_name, new_entry_pathbuf.display(), other);
                }
            };
            let depth = depth.unwrap_or(0);
            match fs::create_dir(&new_entry_pathbuf) {
                Ok(_) => {},
                Err(err) => {
                    return wrap_err!("{} unable to create directory at '{}' due to err: {}", function_name, current_path.display(), err);
                }
            };
            write_tree_rec(new_entry_pathbuf, subtree, Some(depth + 1), function_name)?;
        }
    }
    Ok(())
}

/// fs.removetree(path: string)
/// does NOT follow symlinks
pub fn fs_removetree(_luau: &Lua, value: LuaValue) -> LuaEmptyResult {
    let function_name = "fs.removetree(path: string)";
    let victim_path = match value {
        LuaValue::String(path) => {
            validate_path(&path, function_name)?
        },
        other => {
            return wrap_err!("fs.removetree(path: string) expected path to be a string, got: {:?}", other);
        }
    };
    let metadata = match fs::metadata(&victim_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return wrap_io_read_errors_empty(err, function_name, &victim_path);
        }
    };
    if metadata.is_dir() {
        if let Err(err) = fs::remove_dir_all(&victim_path) {
            let err_message = "fs.removetree was unable to remove some, or all of the directory tree requested:\n";
            wrap_err!("{}    {}", err_message, err)
        } else {
            Ok(())
        }
    } else {
        wrap_err!("fs.removetree(path: string) expected to find a directory at path '{}' but instead found a file", victim_path)
    }
}

/// fs.listdir(path: string, recursive: boolean?): { string }
fn fs_listdir(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let dir_path = match multivalue.pop_front() {
        Some(LuaValue::String(path)) => {
            validate_path(&path, "fs.listdir(path: string, recursive: boolean?)")?
        },
        Some(other) => {
            return wrap_err!("fs.listdir(path: string, recursive: boolean?) expected path to be a string, got: {:#?}", other);
        },
        None => {
            return wrap_err!("fs.listdir(path: string, recursive: boolean?) called without any arguments");
        }
    };
    directory_entry::listdir(luau, dir_path, multivalue, "fs.listdir(path: string, recursive: boolean?)")
}

fn fs_makedir(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let function_name = "fs.makedir(path: string, create_missing: boolean?)";
    let new_path = match multivalue.pop_front() {
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
    let create_missing_dirs = match multivalue.pop_front() {
        Some(LuaValue::Boolean(b)) => b,
        Some(LuaNil) => false,
        Some(other) => {
            return wrap_err!("{}: expected create_missing to be a boolean, nil, or unspecified, got: {:?}", function_name, other);
        },
        None => false,
    };

    match if create_missing_dirs {
        fs::create_dir_all(&new_path)
    } else {
        fs::create_dir(&new_path)
    } {
        Ok(_) => Ok(()),
        Err(err) => {
            match err.kind() {
                io::ErrorKind::AlreadyExists => {
                    wrap_err!("{}: directory '{}' already exists", function_name, &new_path)
                },
                io::ErrorKind::NotFound => {
                    wrap_err!(
                        "{}: path to '{}' not found; pass 'true' as fs.makedir's second argument to create the missing directories,\n\
                        and/or make sure the passed path starts in '/', './', or '../'",
                        function_name, &new_path
                    )
                },
                _ => {
                    wrap_err!("{} unable to create directory/directories due to err: {}", function_name, err)
                }
            }
        }
    }
}

fn fs_entries(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "fs.entries(directory: string)";
    let directory_path = match value {
        LuaValue::String(path) => {
            validate_path(&path, function_name)?
        },
        other => {
            return wrap_err!("{} expected directory to be a string, got: {:?}", function_name, other);
        }
    };
    let metadata = match fs::metadata(&directory_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return wrap_io_read_errors(err, function_name, &directory_path);
        }
    };
    if !metadata.is_dir() {
        return wrap_err!("{} expected '{}' to be a directory, got file instead", function_name, directory_path);
    }

    let mut entry_vec: Vec<(String, LuaValue)> = Vec::new();

    for current_entry in fs::read_dir(&directory_path)? {
        let current_entry = current_entry?;
        let entry_path = current_entry.path().to_string_lossy().to_string();
        // entry::create creates either a FileEntry or DirectoryEntry as needed
        let entry_table = entry::create(luau, &entry_path, function_name)?;
        entry_vec.push((entry_path, entry_table));
    }

    ok_table(TableBuilder::create(luau)?
        .with_values(entry_vec)?
        .build_readonly()
    )
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

/// fs.find(path: string, follow_symlinks: boolean?): FindResult
fn fs_find(luau: &Lua, multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "fs.find(path: string, follow_symlinks: boolean?)";
    find(luau, multivalue, function_name)
}

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
                            let new_entry = entry::create(luau, &entry_path, "FindResult:retry_file()")?;
                            find_result.raw_set("file", new_entry)?;
                            let new_entry: LuaValue = find_result.raw_get("file")?;
                            Ok(new_entry)
                        } else {
                            Ok(LuaNil)
                        }
                    } else {
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

pub fn fs_exists(_luau: &Lua, path: LuaValue) -> LuaValueResult {
    let path = match path {
        LuaValue::String(path) => path.to_string_lossy(),
        other => {
            return wrap_err!("fs.exists(path) expected path to be a string, got: {:#?}", other);
        }
    };

    match fs::exists(&path) {
        Ok(true) => Ok(LuaValue::Boolean(true)),
        Ok(false) => Ok(LuaValue::Boolean(false)),
        Err(err) => {
            match err.kind() {
                io::ErrorKind::PermissionDenied => {
                    wrap_err!("fs.exists: attempt to check if path '{}' exists but permission denied", path)
                },
                other => {
                    wrap_err!("fs.exists: encountered an error checking if '{}' exists: {}", path, other)
                }
            }
        }
    }
}

fn fs_file_from(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let path = match value {
        LuaValue::String(path) => path.to_string_lossy(),
        other => {
            return wrap_err!("fs.file.from(path) expected path to be a string, got: {:#?}", other);
        }
    };
    ok_table(file_entry::create(luau, &path))
}

fn fs_file_build(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let file_name = match multivalue.pop_front() {
        Some(LuaValue::String(name)) => name,
        Some(other) => {
            return wrap_err!("fs.file.build(name: string, content: string) expected name to be a string, got: {:?}", other);
        },
        None => {
            return wrap_err!("fs.file.build(name: string, content: string) expected name, got nothing");
        }
    };
    let file_content = match multivalue.pop_front() {
        Some(LuaValue::String(content)) => content,
        Some(other) => {
            return wrap_err!("fs.file.build(name: string, content: string) expected content to be a string, got: {:?}", other);
        },
        None => {
            return wrap_err!("fs.file.build(name: string, content: string) expected content, got nothing");
        }
    };
    ok_table(TableBuilder::create(luau)?
        .with_value("type", "File")?
        .with_value("name", file_name)?
        .with_value("content", file_content)?
        .build()
    )
}

pub fn create_filelib(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("from", fs_file_from)?
        .with_function("build", fs_file_build)?
        .with_metatable(TableBuilder::create(luau)?
            .with_function("__call", {
                | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                    let function_name = "fs.file:__call(path: string)";
                    let Some(LuaValue::Table(_filelib)) = multivalue.pop_front() else {
                        return wrap_err!("{}: somehow called without self (or where self isn't a table)? this is impossible", function_name);
                    };
                    let file_path = match multivalue.pop_front() {
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
                    let LuaValue::Table(find_result) = fs_find(luau, file_path.into_lua_multi(luau)?)? else {
                        return wrap_err!("[Internal error]: {}: if fs_find doesn't return a table it's gone insane", function_name)
                    };
                    match find_result.raw_get("file")? {
                        LuaValue::Table(t) => ok_table(Ok(t)),
                        LuaNil => Ok(LuaNil),
                        other => {
                            wrap_err!("[Internal error]: {}: find_result.file returned smth that isn't a table nor nil: {:?}", function_name, other)
                        }
                    }
                }
            })?
            .build_readonly()?
        )?
        .build_readonly()
}

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

pub fn create_dirlib(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("from", fs_dir_from)?
        .with_function("build", fs_dir_build)?
        .with_metatable(TableBuilder::create(luau)?
            .with_function("__call", {
                | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                    let function_name = "fs.dir:__call(path: string)";
                    let Some(LuaValue::Table(_filelib)) = multivalue.pop_front() else {
                        return wrap_err!("{}: somehow called without self (or where self isn't a table)? this is impossible", function_name);
                    };
                    let file_path = match multivalue.pop_front() {
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
                    let LuaValue::Table(find_result) = fs_find(luau, file_path.into_lua_multi(luau)?)? else {
                        return wrap_err!("[Internal error]: {}: if fs_find doesn't return a table it's gone insane", function_name)
                    };
                    match find_result.raw_get("dir")? {
                        LuaValue::Table(t) => ok_table(Ok(t)),
                        LuaNil => Ok(LuaNil),
                        other => {
                            wrap_err!("[Internal error]: {}: find_result.dir returned smth that isn't a table nor nil: {:?}", function_name, other)
                        }
                    }
                }
            })?
            .build_readonly()?
        )?
        .build_readonly()
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    let std_fs = TableBuilder::create(luau)?
        .with_function("readfile", fs_readfile)?
        .with_function("readbytes", fs_readbytes)?
        .with_function("readlines", fs_readlines)?
        .with_function("writefile", fs_writefile)?
        .with_function("move", fs_move)?
        .with_function("removefile", fs_removefile) ?
        .with_function("listdir", fs_listdir)?
        .with_function("makedir", fs_makedir)?
        .with_function("writetree", fs_writetree)?
        .with_function("removetree", fs_removetree)?
        .with_function("entries", fs_entries)?
        .with_function("find", fs_find)?
        .with_function("exists", fs_exists)?
        .with_value("path", pathlib::create(luau)?)?
        .with_value("file", create_filelib(luau)?)?
        .with_value("dir", create_dirlib(luau)?)?
        .build_readonly()?;

    Ok(std_fs)
}
