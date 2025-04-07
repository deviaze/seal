use mlua::prelude::*;
use crate::{LuaValueResult, LuaEmptyResult, wrap_err};
use crate::{colors, table_helpers::TableBuilder};
use crate::std_fs::entry::{self, wrap_io_read_errors, wrap_io_read_errors_empty, get_path_from_entry};
use std::cell::RefCell;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::io::{BufRead, BufReader, Read, Seek, Write};

#[cfg(unix)]
use std::os::unix::fs::FileExt;

fn file_readfile(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let file_path = get_path_from_entry(&value, "FileEntry:read()")?;
    let bytes = match fs::read(&file_path) {
        Ok(bytes) => bytes,
        Err(err) => {
            return wrap_io_read_errors(err,"FileEntry:read()", &file_path);
        }
    };
    Ok(LuaValue::String(luau.create_string(bytes)?))
}

/// helper function for fs.readbytes and FileEntry:readbytes(); expects multivalue like
/// (file_offset: number?, count: number?, target_buffer: buffer?, buffer_offset: number?)
pub fn read_file_into_buffer(luau: &Lua, entry_path: &str, mut multivalue: LuaMultiValue, function_name_and_args: &str) -> LuaValueResult {
    let try_truncate_f64 = | f: f64, context: &str | -> LuaResult<i32> {
        let truncated_f = f.trunc();
        if truncated_f != f {
            Ok(truncated_f as i32)
        } else {
            wrap_err!("{} expected {} to be an integer number, got floating point number", function_name_and_args, context)
        }
    };

    let file_offset = match multivalue.pop_front() {
        Some(LuaValue::Integer(n)) => n,
        Some(LuaValue::Number(f)) => try_truncate_f64(f, "file_offset")?,
        Some(LuaNil) | None => 0,
        Some(other) => {
            return wrap_err!("{} expected file_offset to be a number (integer), got: {:#?}", function_name_and_args, other);
        },
    };

    let count = match multivalue.pop_front() {
        Some(LuaValue::Integer(n)) => Some(n),
        Some(LuaValue::Number(f)) => Some(try_truncate_f64(f, "count")?),
        Some(LuaNil) | None => None,
        Some(other) => {
            return wrap_err!("{} expected count to be a number (integer), got: {:#?}", function_name_and_args, other);
        },
    };

    let target_buffer = match multivalue.pop_front() {
        Some(LuaValue::Buffer(buffy)) => Some(buffy),
        Some(LuaNil) | None => None,
        Some(other) => {
            return wrap_err!("{} expected target_buffer to be a buffer, got: {:#?}", function_name_and_args, other)
        },
    };

    let buffer_offset = match multivalue.pop_front() {
        Some(LuaValue::Integer(n)) => n,
        Some(LuaValue::Number(f)) => try_truncate_f64(f, "buffer_offset")?,
        Some(LuaNil) | None => 0,
        Some(other) => {
            return wrap_err!("{} expected buffer_offset to be a number (integer), got: {:#?}", function_name_and_args, other);
        },
    };

    // sanity checks
    let assert_sign = |n: i32, context: &str | -> LuaResult<u64> {
        if n < 0 {
            wrap_err!("{}: {} cannot be negative", function_name_and_args, context)
        } else {
            Ok(n as u64)
        }
    };

    let file_size = match fs::metadata(entry_path) {
        Ok(metadata) => {
            metadata.len()
        },
        Err(err) => {
            return wrap_io_read_errors(err, function_name_and_args, entry_path);
        }
    };

    let buffer_offset = assert_sign(buffer_offset, "buffer_offset")?;
    let file_offset = assert_sign(file_offset, "file_offset")?;
    let count = {
        if let Some(count) = count {
            assert_sign(count, "count")?
        } else {
            file_size
        }
    };

    let buffer_size = {
        if let Some(target_buffer) = &target_buffer {
            target_buffer.len() as u64
        } else {
            count
        }
    };
    
    if (buffer_offset + count) > buffer_size {
        return wrap_err!("{}: target buffer too small! buffer_offset + count is {}, which is larger than the provided buffer ({})", function_name_and_args, buffer_offset + count, buffer_size);
    } else if (file_offset + count) > file_size {
        return wrap_err!("{}: file_offset + count ({}) is greater than the file size ({})", function_name_and_args, file_offset + count, file_size);
    }

    let mut file = match fs::File::open(entry_path) {
        Ok(file) => file,
        Err(err) => {
            return wrap_io_read_errors(err, function_name_and_args, entry_path);
        }
    };

    let count = match count.try_into() {
        Ok(count) => count,
        Err(_err) => {
            return wrap_err!("{}: can't convert u64 ({}) to usize needed to read bytes from file", function_name_and_args, count);
        }
    };

    let mut rust_buffer = vec![0; count];
    #[cfg(unix)]
    {
        if let Err(err) = file.read_at(&mut rust_buffer, file_offset) {
            return wrap_err!("{}: error reading file: {}", function_name_and_args, err);
        }
    }

    #[cfg(windows)]
    {
        use std::io::SeekFrom;

        if let Err(err) = file.seek(SeekFrom::Start(file_offset)) {
            return wrap_err!("{}: error seeking file: {}", function_name_and_args, err);
        }

        if let Err(err) = file.read(&mut rust_buffer) {
            return wrap_err!("{}: error reading file: {}", function_name_and_args, err);
        }
    }

    if let Some(target_buffer) = target_buffer {
        target_buffer.write_bytes(buffer_offset as usize, &rust_buffer);
        Ok(LuaValue::Buffer(target_buffer))
    } else {
        Ok(LuaValue::Buffer(luau.create_buffer(rust_buffer)?))
    }
}

fn file_readbytes(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "FileEntry:readbytes(file_offset: number?, count: number?, target_buffer: buffer?, buffer_offset: number?)";
    let entry = match multivalue.pop_front() {
        Some(value) => value,
        None => {
            return wrap_err!("{} incorrectly called without self, did you forget to use methodcall syntax (:)?", function_name);
        }
    };
    let entry_path = get_path_from_entry(&entry, function_name)?;

    // read_entry_path_into_buffer(luau, entry_path, multivalue, "FileEntry:readbytes")
    read_file_into_buffer(luau, &entry_path, multivalue, function_name)
}

fn file_append(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let entry = match multivalue.pop_front() {
        Some(value) => value,
        None => {
            return wrap_err!("FileEntry:append(content) expected to be called with self but was incorrectly called with zero arguments");
        }
    };

    let entry_path = get_path_from_entry(&entry, "FileEntry:append(content: string | buffer)")?;

    let mut file = match OpenOptions::new()
        .append(true)
        .open(&entry_path) 
    {
        Ok(file) => file,
        Err(err) => {
            return wrap_io_read_errors_empty(err, "FileEntry:append", &entry_path);
        }
    };
    
    let content = match multivalue.pop_front() {
        Some(LuaValue::String(content)) => {
            let content = content.to_string_lossy();
            content.as_bytes().to_owned()
        },
        Some(LuaValue::Buffer(buffy)) => {
            buffy.to_vec()
        },
        Some(other) => {
            return wrap_err!("FileEntry:append(content) expected content to be a string or buffer, got: {:#?}", other);
        },
        None => {
            return wrap_err!("FileEntry:append(content) expected arguments self and content but got no second argument");
        }
    };

    match file.write_all(&content) {
        Ok(_) => Ok(()),
        Err(err) => {
            wrap_err!("FileEntry:append: error writing to file: {}", err)
        }
    }

}

// TODO: investigate whether this is an actually good way of iterating thru lines or whether this is cursed
// something tells me this isn't as performant as it can be
// we can't make this thing return FnMut due to mlua reasons so we have to keep reader and current_line in refcells
pub fn readlines(luau: &Lua, entry_path: &str, function_name: &str) -> LuaValueResult {
    let file = match fs::File::open(entry_path) {
        Ok(file) => file,
        Err(err) => {
            return entry::wrap_io_read_errors(err, function_name, entry_path);
        }
    };

    let function_name = function_name.to_owned();

    let reader = BufReader::new(file);
    let reader_cell = RefCell::new(reader);

    let current_line = 0;
    let current_line_cell = RefCell::new(current_line);

    Ok(LuaValue::Function(luau.create_function({
        move | luau: &Lua, _value: LuaValue | -> LuaResult<LuaMultiValue> {
            let mut reader_cell = reader_cell.borrow_mut();
            let reader = reader_cell.by_ref();
            let mut new_line = String::new();
            match reader.read_line(&mut new_line) {
                Ok(0) => {
                    let multi_vec = vec![LuaNil];
                    Ok(LuaMultiValue::from_vec(multi_vec))
                },
                Ok(_other) => {
                    let mut current_line = current_line_cell.borrow_mut();
                    *current_line += 1;
                    let luau_line = luau.create_string(new_line.trim_end())?;
                    let multi_vec = vec![LuaValue::Integer(*current_line), LuaValue::String(luau_line)];
                    Ok(LuaMultiValue::from_vec(multi_vec))
                },
                Err(err) => {
                    wrap_err!("{}: unable to read line: {}", function_name, err)
                }
            }
        }
    })?))
}

fn file_readlines(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let entry_path = get_path_from_entry(&value, "FileEntry:readlines()")?;
    readlines(luau, &entry_path, "FileEntry:readlines()")
}

fn file_filesize(_luau: &Lua, value: LuaValue) -> LuaValueResult {
    let file_path = get_path_from_entry(&value, "FileEntry:size()")?;
    let metadata = match fs::metadata(&file_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return wrap_io_read_errors(err, "FileEntry:size()", &file_path);
        }
    };
    Ok(LuaValue::Number(metadata.len() as f64))
}

fn file_is_valid_utf8(_luau: &Lua, value: LuaValue) -> LuaValueResult {
    let entry_path = get_path_from_entry(&value, "FileEntry:is_valid_utf8()")?;
    let mut file = match fs::File::open(&entry_path) {
        Ok(file) => file,
        Err(err) => {
            return wrap_io_read_errors(err, "FileEntry:is_valid_utf8()", &entry_path);
        }
    };
    let mut buffer = Vec::new();
    match file.read_to_end(& mut buffer) {
        Ok(_) => {},
        Err(err) => {
            return wrap_err!("FileEntry:is_valid_utf8(): error reading file: {}", err);
        }
    };
    match std::str::from_utf8(&buffer) {
        Ok(_) => Ok(LuaValue::Boolean(true)),
        Err(_) => Ok(LuaValue::Boolean(false)),
    }
}

pub fn create(luau: &Lua, path: &str) -> LuaResult<LuaTable> {
    let original_path = path;
    let path = PathBuf::from(path);
    if !path.exists() || !path.is_file() {
        return wrap_err!("File not found: '{}'", path.display());
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
        .with_value("type", "File")?
        .with_function("size", file_filesize)?
        .with_function("read", file_readfile)?
        .with_function("readbytes", file_readbytes)?
        .with_function("readlines", file_readlines)?
        .with_function("is_valid_utf8", file_is_valid_utf8)?
        .with_function("append", file_append)?
        .with_function("metadata", entry::metadata)?
        .with_function("copy_to", entry::copy_to)?
		.with_function("move_to", entry::move_to)?
		.with_function("rename", entry::rename)?
		.with_function("remove", entry::remove)?
        .build_readonly()
}