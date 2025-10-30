use crate::prelude::*;
use crate::require::get_chunk_name_for_module;
use crate::{globals, std_json};
use mluau::prelude::*;
use petname::Generator;
/// helper struct for ThreadSpawnOptions tables so we don't crowd std_thread
use std::{fs, io, path::PathBuf};

#[derive(Default)]
pub struct ChannelCapacity {
    pub regular: usize = 12,
    pub bytes: usize = 24,
}
pub struct ThreadSpawnOptions {
    pub name: String,
    pub chunk_name: String,
    pub spawned_at: String,
    pub capacity: ChannelCapacity,
    pub src: Option<String>,
    pub path: Option<PathBuf>,
    pub data: Option<String>,
}

impl ThreadSpawnOptions {
    pub fn from_table(t: LuaTable, luau: &Lua, function_name: &'static str) -> LuaResult<Self> {
        let name = match t.raw_get("name")? {
            LuaValue::String(name) => name.to_string_lossy(),
            LuaNil => {
                let Some(petname) = ({
                    let alliterations = petname::Alliterations::default();
                    alliterations.generate_one(3, "-")
                }) else {
                    panic!(
                        "{}: unable to generate petname for ThreadSpawnOptions.name; this panic might seem ridiculous and this shouldn't happen lol",
                        function_name
                    );
                };
                petname
            }
            other => {
                return wrap_err!("{}: ThreadSpawnOptions.name expected to be string or nil, got: {:?}", function_name, other);
            }
        };
        let src = match t.raw_get("src")? {
            LuaValue::String(src) => Some(src.to_string_lossy()),
            LuaNil => None,
            other => {
                return wrap_err!("{}: ThreadSpawnOptions.src expected to be a string, got: {:?}", function_name, other);
            }
        };
        let path = match t.raw_get::<LuaValue>("path")? {
            LuaValue::String(path) => {
                // need to make sure path is relative to the calling script's path
                let LuaValue::String(parent_path) = globals::get_script_parent(luau, LuaMultiValue::new())? else {
                    return wrap_err!("{}: unable to get script's parent path", function_name);
                };
                let path = PathBuf::from(path.to_string_lossy());
                let parent_path = {
                    let parent = PathBuf::from(parent_path.to_string_lossy());
                    if path.starts_with("..")
                        && let Some(parent_parent) = parent.parent()
                    {
                        parent_parent.to_path_buf()
                    } else {
                        parent
                    }
                };
                Some(parent_path.join(path))
            }
            LuaNil => None,
            other => {
                return wrap_err!("{}: ThreadSpawnOptions.path expected to be a string, got: {:?}", function_name, other);
            }
        };
        if src.is_none() && path.is_none() {
            return wrap_err!("{}: ThreadSpawnOptions must have either fields 'path' or 'src', got neither", function_name);
        }
        let data = match t.raw_get("data")? {
            LuaNil => None,
            LuaValue::Table(data_table) => Some(std_json::encode(luau, data_table, std_json::EncodeOptions { pretty: false, sorted: false })?),
            LuaValue::String(s) => Some(s.to_string_lossy()),
            other => {
                return wrap_err!("{}: ThreadSpawnOptions.data expected to be a table, string, or nil, got: {:?}", function_name, other);
            }
        };
        let capacity = match t.raw_get("capacity")? {
            LuaNil => ChannelCapacity::default(),
            LuaValue::Table(capacity_table) => {
                let regular: usize = match capacity_table.raw_get("regular")? {
                    LuaNil => 12,
                    LuaValue::Integer(i) => int_to_usize(i, function_name, "capacity.regular")?,
                    other => {
                        return wrap_err!("{}: ThreadSpawnOptions.capacity.regular expected to be a number (integer) or nil, got: {:?}", function_name, other);
                    }
                };
                let bytes: usize = match capacity_table.raw_get("bytes")? {
                    LuaNil => 24,
                    LuaValue::Integer(i) => int_to_usize(i, function_name, "capacity.bytes")?,
                    other => {
                        return wrap_err!("{}: ThreadSpawnOptions.capacity.bytes expected to be a number (integer) or nil, got: {:?}", function_name, other);
                    }
                };
                ChannelCapacity { regular, bytes }
            }
            other => {
                return wrap_err!(
                    "{}: expected ThreadSpawnOptions.capacity to be an optional table with fields regular and bytes, got: {:?}",
                    function_name,
                    other
                );
            }
        };
        let chunk_name = {
            if let Some(ref path) = path
                && let Some(path) = path.to_str()
                && let Some(chunk_name) = get_chunk_name_for_module(path, function_name)?
            {
                chunk_name
            } else {
                name.clone()
            }
        };
        let spawned_at = {
            let debug_info = DebugInfo::from_caller(luau, function_name)?;
            format!("{}:{} in {}", debug_info.source, debug_info.line, debug_info.function_name)
        };

        Ok(Self {
            name,
            spawned_at,
            chunk_name,
            src,
            path,
            capacity,
            data,
        })
    }
    pub fn get_src(&self, function_name: &'static str) -> LuaResult<String> {
        if let Some(ref src) = self.src {
            return Ok(src.to_owned());
        }
        let Some(ref src_path) = self.path else {
            return wrap_err!("{}: ThreadSpawnOptions missing either field 'path' or 'src'", function_name);
        };
        let bytes = match fs::read(src_path) {
            Ok(bytes) => bytes,
            Err(err) => {
                return {
                    match err.kind() {
                        io::ErrorKind::NotFound => wrap_err!("{}: File/directory not found: '{}'", function_name, src_path.display()),
                        io::ErrorKind::PermissionDenied => wrap_err!("{}: Permission denied: '{}'", function_name, src_path.display()),
                        _ => wrap_err!("{}: Error on path: '{}': {}", function_name, src_path.display(), err)
                        
                    }
                };
            }
        };
        if let Ok(src) = String::from_utf8(bytes) {
            Ok(src)
        } else {
            wrap_err!("{}: unable to convert src from path '{}' to valid utf8", function_name, src_path.display())
        }
    }
}
