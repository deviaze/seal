use entry::{wrap_io_read_errors, wrap_io_read_errors_empty};
use mluau::prelude::*;
use crate::prelude::*;
use std::path::{Path, PathBuf};
use std::{fs, io};
use copy_dir::copy_dir;

pub mod entry;
pub mod dirlib;
pub mod pathlib;
pub mod filelib;
pub mod file_entry;
pub mod directory_entry;
pub mod find;

/// helper and converter function to turn LuaStrings into Rust Strings
/// use this one if we're okay with checking the filesystem for common issues for better user experience,
/// don't use this one if we expect to handle permission denied without erroring
pub fn validate_path(path: &LuaString, function_name: &str) -> LuaResult<String> {
    let Ok(path) = path.to_str() else {
        return wrap_err!("{}: provided path '{}' is not properly utf8-encoded", function_name, path.display());
    };
    let path = path.to_string();
    if cfg!(target_os="linux") {
        let path_exists = match fs::exists(&path) {
            Ok(b) => b,
            Err(err) => match err.kind() {
                io::ErrorKind::PermissionDenied => {
                    return wrap_err!("{}: permission denied when trying to validate path at '{}'; if this function isn't supposed to error when PermissionDenied, you've found a bug", function_name, path);
                },
                _ => {
                    return wrap_err!("{}: unexpected error when validating path: {}", function_name, path);
                }
            }
        };
        if !path_exists && fs::exists("/".to_string() + &path)? {
            return wrap_err!("{}: provided path '{}' doesn't exist but an absolute path of it ('/{}') does; did you mean to prepend '/' to it?", function_name, &path, &path);
        } else if !path_exists && path.starts_with("home") { // /home/user/ is ~ on linux
            return wrap_err!("{}: path '{}' looks like an absolute path but doesn't start with '/', did you mean to provide an absolute path?", function_name, &path);
        }
    }
    Ok(path)
}

pub fn validate_path_without_checking_fs(path: &LuaString, function_name: &str) -> LuaResult<String> {
    let Ok(path) = path.to_str() else {
        return wrap_err!("{}: provided path '{}' is not properly utf8-encoded", function_name, path.display());
    };
    Ok(path.to_string())
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
pub fn fs_readbytes(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
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
    file_entry::read_file_into_buffer(luau, &entry_path, multivalue, function_name_and_args)
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
                    wrap_err!("fs.writefile: path to '{}' doesn't exist, are all directories present and does the path start with '/', './', or '../'?", file_path)
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

pub fn fs_copy(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let function_name = "fs.copy(source: string, destination: string)";
    let source_path = match multivalue.pop_front() {
        Some(LuaValue::String(path)) => {
            validate_path(&path, function_name)?
        }
        Some(other) => {
            return wrap_err!("{} expected source path to be a string, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected source, got nothing", function_name);
        }
    };
    let destination_path = match multivalue.pop_front() {
        Some(LuaValue::String(path)) => {
            validate_path(&path, function_name)?
        },
        Some(other) => {
            return wrap_err!("{} expected destination path to be a string, got: {:?}", function_name, other);
        }
        None => {
            return wrap_err!("{} expected destination, got nothing", function_name);
        }
    };
    let source_pathbuf = PathBuf::from(&source_path);
    let mut destination_pathbuf = PathBuf::from(&destination_path);
    
    if source_pathbuf.is_file() && destination_pathbuf.is_dir() {
        // copying a file into a directory shouldn't require you to type the filename again
        let source_filename = match source_pathbuf.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => {
                return wrap_err!("{} source path doesn't have a basename?", function_name);
            }
        };
        destination_pathbuf.push(source_filename);
    } else if source_pathbuf.is_dir() && destination_pathbuf.is_file() {
        return wrap_err!("{}: attempt to copy directory '{}' into file '{}'", function_name, source_path, destination_path);
    }

    let source_to_destination = format!("{} -> {}", source_pathbuf.display(), destination_pathbuf.display());
    if source_pathbuf.is_file() {
        match fs::copy(&source_pathbuf, &destination_pathbuf) {
            Ok(_) => Ok(()),
            Err(err) => {
                wrap_err!("{} unable to copy {} due to err {}", function_name, source_to_destination, err)
            }
        }
    } else {
        match copy_dir(&source_pathbuf, &destination_pathbuf) {
            Ok(unsuccessful) => {
                if !unsuccessful.is_empty() {
                    eprintln!("{} didn't fully succeed:", function_name);
                    for err in unsuccessful {
                        eprintln!("  {}", err);
                    }
                }
                Ok(())
            },
            Err(err) => {
                wrap_io_read_errors_empty(err, function_name, &source_to_destination)
            }
        }
    }
}

const READ_TREE_SRC: &str = include_str!("./read_tree.luau");
/// fs.readtree(path: string): DirectoryTree
/// not called readdir because it's uglier + we want dir/tree stuff to autocomplete after file
/// so we want fs.readfile to autocomplete first and i'm assuming it's alphabetical
fn fs_readtree(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "fs.readtree(path: string)";
    let path = match value {
        LuaValue::String(path) => {
            validate_path(&path, function_name)?
        },
        other => {
            return wrap_err!("{} expected path to be a string, got: {:?}", function_name, other);
        }
    };
    let read_tree_fn: LuaFunction = luau.load(READ_TREE_SRC).eval()?;
    let result = match read_tree_fn.call::<LuaValue>(path) {
        Ok(LuaValue::Table(t)) => t,
        Ok(other) => {
            return wrap_err!("{} [INTERNAL]: read_tree_fn returned something that isn't a table: {:?}", function_name, other);
        }
        Err(err) => {
            return wrap_err!("{}: hit error calling readtree: {}", function_name, err);
        }
    };
    if let LuaValue::Table(directory_tree) = result.raw_get("tree")? {
        Ok(LuaValue::Table(directory_tree))
    } else if let LuaValue::String(err) = result.raw_get("err")? {
        let err = err.to_string_lossy();
        wrap_err!("{}: {}", function_name, err)
    } else {
        wrap_err!("{} [INTERNAL]: read_tree_fn should have returned a table with 'tree' or 'err'???")
    }
}

/// fs.writetree(path: string, tree: TreeBuilder | DirectoryTree): ()
fn fs_writetree(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let function_name = "fs.writetree(path: string, tree: TreeBuilder | DirectoryTree)";
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
        Some(LuaValue::Table(tree)) => tree,
        Some(LuaNil) | None => {
            return wrap_err!("{} expected tree, got nothing or nil; to create an empty directory, use fs.makedir or pass an empty table as tree", function_name);
        }
        Some(other) => {
            return wrap_err!("{} expected tree to be a DirectoryTree (use fs.dir.build to create DirectoryTrees), got: {:?}", function_name, other);
        },
    };

    writetree(luau, path, tree, function_name)
}

pub fn writetree(_luau: &Lua, path: String, tree: LuaTable, function_name: &str) -> LuaEmptyResult {
    let tree = {
        // shadow tree if TableBuilder passed instead of DirectoryTree
        let tree = match tree.raw_get("inner")? {
            LuaValue::Table(inner) => inner,
            LuaNil => tree,
            other => {
                return wrap_err!("{} expected tree to be a TreeBuilder (passed table has key 'inner') but 'inner' is not a table and unexpectedly {:?}", function_name, other);
            }
        };
        match tree.raw_get("type")? {
            LuaValue::String(_) => {
                return wrap_err!("{} expected tree to be a TreeBuilder, or an array-like table which contains entries from fs.dir.build or fs.file.build; did you accidentally pass fs.dir.build to fs.writetree directly?", function_name);
            },
            LuaNil => {},
            _other => {
                return wrap_err!("{} expected tree to be a TreeBuilder or an array-like table, got an invalid table", function_name);
            }
        };
        match tree.raw_get(1)? {
            LuaValue::Table(_) | LuaNil => tree, // allows fs.writetree/DirectoryEntry:add_tree with empty table or empty fs.tree()
            other => {
                return wrap_err!("{} expected tree to be an array-like table (that contains entries from fs.file.build or fs.dir.build), got: {:?}", function_name, other);
            }
        }
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


fn fs_treebuilder_with_file(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "TreeBuilder:with_file(name: string, content: string)";
    let treebuilder = match multivalue.pop_front() {
        Some(LuaValue::Table(treebuilder)) => treebuilder,
        Some(other) => {
            return wrap_err!("{} expected self to be a TreeBuilder, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected to be called with self (methodcall syntax); did you forget a ':'?", function_name);
        }
    };
    let name = match multivalue.pop_front() {
        Some(LuaValue::String(name)) => name,
        Some(other) => {
            return wrap_err!("{} expected name to be a string, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected name, got nothing", function_name);
        }
    };
    let content = match multivalue.pop_front() {
        Some(LuaValue::String(content)) => content,
        Some(other) => {
            return wrap_err!("{} expected content to be a string, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected content, got nothing", function_name);
        }
    };
    let inner = match treebuilder.raw_get("inner")? {
        LuaValue::Table(t) => t,
        other => {
            return wrap_err!("{}: expected self.inner to be a table, got: {:?}; why did you modify it??", function_name, other);
        }
    };
    let filebuilder = TableBuilder::create(luau)?
        .with_value("type", "File")?
        .with_value("name", name)?
        .with_value("content", content)?
        .build_readonly()?;
    inner.raw_push(filebuilder)?;

    Ok(LuaValue::Table(treebuilder))
}

/// TreeBuilder:with_dir(name: string, builder: TreeBuilder)
/// used to construct trees with builder pattern by appending the inner of the passed builder to the TreeBuilder's inner
fn fs_treebuilder_with_tree(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "TreeBuilder:with_tree(name: string, builder: TreeBuilder)";
    let treebuilder = match multivalue.pop_front() {
        Some(LuaValue::Table(treebuilder)) => treebuilder,
        Some(other) => {
            return wrap_err!("{} expected self to be a TreeBuilder, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected to be called with self (methodcall syntax); did you forget a ':'?", function_name);
        }
    };
    let name = match multivalue.pop_front() {
        Some(LuaValue::String(name)) => name,
        Some(other) => {
            return wrap_err!("{} expected name to be a string, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected name, got nothing", function_name);
        }
    };
    let subtree_inner = match multivalue.pop_front() {
        Some(LuaValue::Table(builder)) => {
            match builder.raw_get("inner")? {
                LuaValue::Table(inner) => inner,
                other => {
                    return wrap_err!("{} expected builder to be a TreeBuilder from fs.tree(), got: {:?}; did you pass an unrelated table instead?", function_name, other);
                }
            }
        },
        Some(other) => {
            return wrap_err!("{} expected builder to be a table (a TableBuilder from fs.tree()), got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected builder to be a table (a TableBuilder from fs.tree()), got nothing");
        }
    };
    let inner = match treebuilder.raw_get("inner")? {
        LuaValue::Table(t) => t,
        other => {
            return wrap_err!("{}: expected self.inner to be a table, got: {:?}; why did you modify it??", function_name, other);
        }
    };
    let directory_builder = TableBuilder::create(luau)?
        .with_value("type", "Directory")?
        .with_value("name", name)?
        .with_value("children", subtree_inner)?
        .build_readonly()?;
    inner.raw_push(directory_builder)?;

    Ok(LuaValue::Table(treebuilder))
}

fn fs_tree(luau: &Lua, _value: LuaValue) -> LuaValueResult {
    ok_table(TableBuilder::create(luau)?
        .with_value("inner", luau.create_table()?)?
        .with_function("with_file", fs_treebuilder_with_file)?
        .with_function("with_tree", fs_treebuilder_with_tree)?
        .build_readonly()
    )
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
    if let Err(err) = fs::remove_dir_all(&victim_path) {
        wrap_err!("fs.removetree was unable to remove some (or all) of the directory at '{}' due to err: {}", victim_path, err)
    } else {
        Ok(())
    }
}

/// fs.listdir(path: string, options: { recursive: boolean?, relative_to: ("cwd" | "root" | "self")? }?): { string }
fn fs_listdir(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "fs.listdir(path: string, recursive: boolean?)";
    let dir_path = match multivalue.pop_front() {
        Some(LuaValue::String(path)) => {
            validate_path(&path, function_name)?
        },
        Some(other) => {
            return wrap_err!("{} expected path to be a string, got: {:#?}", function_name, other);
        },
        None => {
            return wrap_err!("{} called without any arguments", function_name);
        }
    };
    directory_entry::listdir(luau, dir_path, multivalue, function_name)
}

fn fs_makedir(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "fs.makedir(path: string, options: { create_missing: boolean?, error_if_exists: boolean? }?)";
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

    let options = match multivalue.pop_front() {
        Some(LuaValue::Table(options)) => Some(options),
        Some(LuaNil) | None => None,
        Some(other) => {
            return wrap_err!("{} expected options to be a table, got: {:?}", function_name, other);
        }
    };

    let create_missing_dirs = if let Some(ref options) = options {
        match options.raw_get("create_missing")? {
            LuaValue::Boolean(b) => b,
            LuaNil => false,
            other => {
                return wrap_err!("{} expected options.create_missing to be an optional boolean (default false), got: {:?}", function_name, other);
            }
        }
    } else {
        false
    };
    let error_if_exists = if let Some(options) = options {
        match options.raw_get("error_if_exists")? {
            LuaValue::Boolean(b) => b,
            LuaNil => true,
            other => {
                return wrap_err!("{} expected options.error_if_exists to be an optional boolean (default true), got: {:?}", function_name, other);
            }
        }
    } else {
        true
    };

    match if create_missing_dirs {
        fs::create_dir_all(&new_path)
    } else {
        fs::create_dir(&new_path)
    } {
        Ok(_) => Ok(LuaValue::Boolean(true)),
        Err(err) => {
            match err.kind() {
                io::ErrorKind::AlreadyExists => {
                    if error_if_exists {
                        wrap_err!("{}: directory '{}' already exists", function_name, &new_path)
                    } else {
                        Ok(LuaValue::Boolean(false))
                    }
                },
                io::ErrorKind::NotFound => {
                    wrap_err!(
                        "{}: path to '{}' not found; pass {{ create_missing = true }} as fs.makedir's second argument to create the missing directories,\n\
                        and/or make sure the passed path starts in '/', './', or '../'",
                        function_name, &new_path
                    )
                },
                io::ErrorKind::PermissionDenied => {
                    wrap_err!("{}: permission denied at '{}'", function_name, &new_path)
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
    entries(luau, value, function_name)
}

pub fn entries(luau: &Lua, value: LuaValue, function_name: &str) -> LuaValueResult {
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

/// fs.find(path: string, follow_symlinks: boolean?): FindResult
fn fs_find(luau: &Lua, multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "fs.find(path: string, follow_symlinks: boolean?)";
    find::find(luau, multivalue, function_name)
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

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    let std_fs = TableBuilder::create(luau)?
        .with_function("readfile", fs_readfile)?
        .with_function("readbytes", fs_readbytes)?
        .with_function("readlines", fs_readlines)?
        .with_function("writefile", fs_writefile)?
        .with_function("move", fs_move)?
        .with_function("copy", fs_copy)?
        .with_function("removefile", fs_removefile) ?
        .with_function("listdir", fs_listdir)?
        .with_function("makedir", fs_makedir)?
        .with_function("readtree", fs_readtree)?
        .with_function("tree", fs_tree)?
        .with_function("writetree", fs_writetree)?
        .with_function("removetree", fs_removetree)?
        .with_function("entries", fs_entries)?
        .with_function("find", fs_find)?
        .with_function("exists", fs_exists)?
        .with_value("path", pathlib::create(luau)?)?
        .with_value("file", filelib::create(luau)?)?
        .with_value("dir", dirlib::create(luau)?)?
        .build_readonly()?;

    Ok(std_fs)
}
