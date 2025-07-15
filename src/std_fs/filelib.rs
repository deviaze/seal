use std::io;
use std::fs::{self, OpenOptions};
use mluau::prelude::*;
use crate::prelude::*;
use super::{
    entry::{self, wrap_io_read_errors},
    file_entry,
    validate_path,
    validate_path_without_checking_fs
};

#[cfg(unix)]
use std::os::unix::fs::FileExt;

#[cfg(windows)]
use std::io::{Seek, Read};

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

fn fs_file_call(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
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

    let metadata = match fs::metadata(&file_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            match err.kind() {
                io::ErrorKind::NotFound => {
                    return Ok(LuaNil)
                },
                io::ErrorKind::PermissionDenied => {
                    return wrap_err!("{}: permission denied at '{}'", function_name, file_path);
                },
                _other => {
                    return wrap_err!("{}: unexpected error at '{}', err: {}", function_name, file_path, err);
                }
            }
        }
    };

    if metadata.is_file() {
        entry::create(luau, &file_path, function_name)
    } else {
        Ok(LuaNil)
    }
}

/// fs.file.create(path: string): FileEntry
/// Creates a new file at path in a TOCTOU (Time of Check to Time Of Use)-compliant manner,
/// note that ONLY the file creation is TOCTOU safe, using the result FileEntry is 100% not TOCTOU safe
fn fs_file_create(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "fs.file.create(path: string)";
    let path = match value {
        LuaValue::String(path) => {
            validate_path(&path, function_name)?
        },
        other => {
            return wrap_err!("{} expected path to be a string, got: {:?}", function_name, other);
        }
    };
    match OpenOptions::new()
        .write(true)
        .create_new(true) // ensure new file is created (TOCTOU)
        .open(&path)
    {
        Ok(_file) => {
            entry::create(luau, &path, function_name)
        },
        Err(err) => {
            wrap_io_read_errors(err, function_name, &path)
        }
    }
}

/// fs.file.try_read(path: string): (content: string?, result: "Ok" | "NotFound" | "PermissionDenied")
/// relatively error-safe and TOCTOU-safe variant of fs.readfile
fn fs_file_try_read(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaMultiResult {
    let function_name = "fs.file.try_read(path: string)";
    let path = match multivalue.pop_front() {
        Some(LuaValue::String(path)) => {
            validate_path_without_checking_fs(&path, function_name)?
        },
        Some(LuaNil) | None => {
            // just because we're 'relatively' error safe doesn't mean we're error safe from bad arguments
            return wrap_err!("{} expected path to be a string, got nil or nothing", function_name);
        },
        Some(other) => {
            return wrap_err!("{} expected path to be a string, got: {:?}", function_name, other);
        }
    };

    let (bytes, result): (Option<Vec<u8>>, &str) = match fs::read(&path) {
        Ok(bytes) => (Some(bytes), "Ok"),
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => (None, "NotFound"),
            io::ErrorKind::PermissionDenied => (None, "PermissionDenied"),
            other => (None, &other.to_string())
        }
    };

    let mut result_multivalue = LuaMultiValue::new();
    if let Some(bytes) = bytes {
        let bytes_string = luau.create_string(&bytes)?;
        result_multivalue.push_back(LuaValue::String(bytes_string));
    } else {
        result_multivalue.push_back(LuaNil);
    }
    result_multivalue.push_back(LuaValue::String(luau.create_string(result)?));

    Ok(result_multivalue)
}

fn fs_file_try_readbytes(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaMultiResult {
    let function_name_and_args = "fs.file.try_readbytes(path: string, file_offset: number?, count: number?, target_buffer: buffer?, buffer_offset: number?)";
    let path = match multivalue.pop_front() {
        Some(LuaValue::String(path)) => {
            validate_path_without_checking_fs(&path, function_name_and_args)?
        },
        Some(LuaNil) | None => {
            return wrap_err!("{} expected path to be a string, got nil or nothing", function_name_and_args);
        },
        Some(other) => {
            return wrap_err!("{} expected path to be a string, got: {:?}", function_name_and_args, other);
        }
    };

    let (file_size, result): (Option<u64>, &str) = match fs::metadata(&path) {
        Ok(metadata) => {
            (Some(metadata.len()), "Ok")
        },
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => (None, "NotFound"),
            io::ErrorKind::PermissionDenied => (None, "PermissionDenied"),
            other => (None, &other.to_string()),
        }
    };

    if file_size.is_none() {
        let mut result_multivalue = LuaMultiValue::new();
        result_multivalue.push_back(LuaNil);
        result_multivalue.push_back(ok_string(result, luau)?);
        // early return if we find an issue like NotFound/PermissionDenied so we don't have to redo those checks later
        return Ok(result_multivalue)
    }

    let file_size = match file_size {
        Some(size) => size,
        None => {
            unreachable!("{}: file size should never be None here because of the above check", function_name_and_args)
        }
    };

    // just copied read_file_into_buffer because i don't want to think of an abstraction for it rn
    let (buffy, result): (Option<mluau::Buffer>, &str) = {
        let try_truncate_f64 = | f: f64, context: &str | -> LuaResult<i64> {
            let truncated_f = f.trunc();
            if truncated_f != f {
                Ok(truncated_f as i64)
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
        let assert_sign = | n: i64, context: &str | -> LuaResult<u64> {
            if n < 0 {
                wrap_err!("{}: {} cannot be negative", function_name_and_args, context)
            } else {
                Ok(n as u64)
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
    
        #[allow(unused_mut, reason = "needs to be mut on windows")]
        let mut file = match fs::File::open(&path) {
            Ok(file) => file,
            Err(err) => {
                return wrap_err!("{}: unexpected error opening file at '{}': {}", function_name_and_args, path, err);
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
            (Some(target_buffer), "Ok")
        } else {
            // Ok(LuaValue::Buffer(luau.create_buffer(rust_buffer)?))
            let luau_buffer = luau.create_buffer(rust_buffer)?;
            (Some(luau_buffer), "Ok")
        }
    };

    let mut result_multivalue = LuaMultiValue::new();
    if let Some(buffy) = buffy {
        result_multivalue.push_back(LuaValue::Buffer(buffy));
    } else {
        result_multivalue.push_back(LuaNil);
    }
    result_multivalue.push_back(ok_string(result, luau)?);
    Ok(result_multivalue)
}

fn fs_file_try_write(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaMultiResult {
    let function_name = "fs.file.try_write(path: string, content: string | buffer";
    let path = match multivalue.pop_front() {
        Some(LuaValue::String(path)) => {
            validate_path_without_checking_fs(&path, function_name)?
        },
        Some(LuaNil) | None => {
            return wrap_err!("{} expected path to be a string, got nil or nothing", function_name);
        },
        Some(other) => {
            return wrap_err!("{} expected path to be a string, got: {:?}", function_name, other);
        }
    };

    let content = match multivalue.pop_front() {
        Some(LuaValue::String(content)) => {
            content.as_bytes().to_vec()
        },
        Some(LuaValue::Buffer(buffy)) => {
            buffy.to_vec()
        },
        Some(LuaNil) | None => {
            return wrap_err!("{} expected content to be a string or buffer, got: nil or nothing", function_name);
        }
        Some(other) => {
            return wrap_err!("{} expected content to be a string or buffer, got: {:?}", function_name, other);
        }
    };

    let (success, result): (bool, &str) = match fs::write(&path, &content) {
        Ok(_) => (true, "Ok"),
        Err(err) => match err.kind() {
            io::ErrorKind::PermissionDenied => (false, "PermissionDenied"),
            other => (false, &other.to_string()),
        }
    };

    let mut result_multivalue = LuaMultiValue::new();
    result_multivalue.push_back(LuaValue::Boolean(success));
    result_multivalue.push_back(ok_string(result, luau)?);
    Ok(result_multivalue)
}

fn fs_file_try_remove(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaMultiResult {
    let function_name = "fs.file.try_remove(path: string)";
    let path = match multivalue.pop_front() {
        Some(LuaValue::String(path)) => {
            validate_path_without_checking_fs(&path, function_name)?
        },
        Some(other) => {
            return wrap_err!("{} expected path to be a string, got: {:?}", function_name, other);
        },
        None => {
            return wrap_err!("{} expected path to be a string, but was called with no arguments", function_name);
        }
    };
    let (success, result): (bool, &str) = match fs::remove_file(&path) {
        Ok(_) => (true, "Ok"),
        Err(err) => match err.kind() {
            io::ErrorKind::PermissionDenied => (false, "PermissionDenied"),
            io::ErrorKind::NotFound => (false, "NotFound"),
            io::ErrorKind::IsADirectory => (false, "IsADirectory"),
            other => (false, &other.to_string()),
        }
    };
    let result_multi = LuaMultiValue::from_vec(vec![
        LuaValue::Boolean(success),
        LuaValue::String(luau.create_string(result)?)
    ]);
    Ok(result_multi)
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("from", fs_file_from)?
        .with_function("build", fs_file_build)?
        .with_function("create", fs_file_create)?
        .with_function("try_read", fs_file_try_read)?
        .with_function("try_readbytes", fs_file_try_readbytes)?
        .with_function("try_write", fs_file_try_write)?
        .with_function("try_remove", fs_file_try_remove)?
        .with_metatable(TableBuilder::create(luau)?
            .with_function("__call", fs_file_call)?
            .build_readonly()?
        )?
        .build_readonly()
}