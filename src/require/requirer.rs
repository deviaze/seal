#![allow(unused)] //  haven't switched to the new requirer yet 

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

use mlua::prelude::*;
use mlua::Require;

use crate::std_json::json_decode;
use crate::{colors, LuaValueResult};

#[derive(Debug)]
pub struct FsRequirer {
    // luau: &'a Lua,
    require_cache: LuaTable,
    project_path: RefCell<PathBuf>,
    current_path: RefCell<PathBuf>,
    current_chunk: RefCell<PathBuf>,
}

impl FsRequirer {
    pub fn from(cwd: PathBuf, current_chunk: PathBuf, cache: LuaTable) -> Self {
        let current_path = current_chunk.parent().expect("can't get current chunk's parent");
        FsRequirer { 
            require_cache: cache,
            project_path: RefCell::from(cwd),
            current_path: RefCell::from(current_path.to_path_buf()), 
            current_chunk: RefCell::from(current_chunk),
        }
    }
    pub fn to_luaurc(&self) {
        while !self.is_config_present() {
            let _ = self.to_parent();
        }
    }
    pub fn read_luaurc(&self) -> LuaResult<String> {
        let current_path = self.current_path.borrow();
        let original_current_path = current_path.clone();
        drop(current_path);
        while !self.is_config_present() {
            let _ = self.to_parent();
        }
        match self.config() {
            Ok(config) => {
                let mut current_path = self.current_path.borrow_mut();
                *current_path = original_current_path;
                match String::from_utf8(config) {
                    Ok(s) => Ok(s),
                    Err(err) => {
                        wrap_err!("can't convert contents of .luaurc to valid utf-8: {}", err)
                    }
                }
            },
            Err(err) => {
                wrap_err!("cant be decoded huhk: {}", err)
            }
        }
    }
    pub fn get_aliases(&self) -> LuaResult<HashMap<String, String>> {
        let luaurc_contents = match self.read_luaurc() {
            Ok(contents) => contents,
            Err(_err) => {
                return Ok(HashMap::new());
            }
        };
        let json_value: serde_json_lenient::Value = match serde_json_lenient::from_str_lenient(&luaurc_contents) {
            Ok(v) => v,
            Err(err) => {
                return wrap_err!("get_aliases can't read .luaurc from json: {}", err);
            }
        };
        if let Some(aliases) = json_value.get("aliases") {
            let Some(aliases) = aliases.as_object() else {
                return wrap_err!(".luaurc.aliases must be a json map/object (with keys and values)");
            };
            let mut result_map = HashMap::new();
            for (key, value) in aliases {
                let Some(s) = value.as_str() else {
                    return wrap_err!("all key/value pairs in .luaurc.aliases must be strings");
                };
                result_map.insert(key.to_owned(), s.to_owned());
            }
            Ok(result_map)
        } else {
            Ok(HashMap::new())
        }
    }
    pub fn show_current_path(&self) -> String {
        self.current_path.borrow().display().to_string()
    }
    pub fn to_project(&self) -> Result<(), LuaNavigateError> {
        let project_path = self.project_path.borrow();
        self.current_path.replace(project_path.to_path_buf());
        Ok(())
    }
    fn is_cached(&self, chunk_name: &str) -> bool {
        match self.require_cache.raw_get(chunk_name) {
            Ok(LuaNil) => false,
            Ok(_other) => true,
            Err(_err) => false,
        }
    }
    pub fn load(&self, luau: &Lua) -> LuaValueResult {
        let require_cache = &self.require_cache;
        let chunk_name = self.chunk_name();
        match require_cache.raw_get(chunk_name)? {
            LuaNil => {
                // let contents = self.contents();
                let contents = match self.contents() {
                    Ok(contents) => match String::from_utf8(contents)  {
                        Ok(contents) => contents,
                        Err(err) => {
                            return wrap_err!("can't decode from utf8: {}", err);
                        }
                    },
                    Err(err) => {
                        return wrap_err!("can't read from file: {}", err);
                    }
                };
                luau.load(&contents).set_name(self.chunk_name()).eval::<LuaValue>()
            },
            other => Ok(other)
        }
    }
}

impl Require for FsRequirer {
    fn is_require_allowed(&self, _chunk_name: &str) -> bool {
        true // why wouldn't it be??
    }
    fn is_config_present(&self) -> bool {
        let current_path = self.current_path.borrow();
        let luaurc_path = current_path.join(".luaurc");
        luaurc_path.is_file()
    }
    fn config(&self) -> std::io::Result<Vec<u8>> {
        let current_path = self.current_path.borrow();
        let luaurc_path = current_path.join(".luaurc");
        fs::read(&luaurc_path)
    }
    fn reset(&self, chunk_name: &str) -> Result<(), LuaNavigateError> {
        let chunk_path = PathBuf::from(chunk_name);
        if let Some(chunk_parent_path) = chunk_path.clone() .parent() {
            // let mut current_chunk = self.current_chunk.borrow_mut();
            // let mut current_path = self.current_path.borrow_mut();
            // *current_chunk = chunk_path.clone();
            // *current_path = chunk_parent_path.to_path_buf();
            self.current_chunk.replace(chunk_path);
            self.current_path.replace(chunk_parent_path.to_path_buf());
            Ok(())
        } else {
            Err(LuaNavigateError::NotFound)
        }
    }
    fn jump_to_alias(&self, path: &str) -> Result<(), LuaNavigateError> {
        println!("jump_to_alias called with {}", path);
        Ok(())
    }
    fn to_parent(&self) -> Result<(), LuaNavigateError> {
        let mut current_path = self.current_path.borrow_mut();
        if let Some(parent_path) = current_path.clone().parent() {
            *current_path = parent_path.to_path_buf();
            Ok(())
        } else {
            Err(LuaNavigateError::NotFound)
        }
    }
    fn to_child(&self, name: &str) -> Result<(), LuaNavigateError> {
        let mut current_path = self.current_path.borrow_mut();
        let child_path = current_path.join(name);
        if child_path.is_dir() {
            let with_init_luau = child_path.join("init.luau");
            if with_init_luau.is_file() {
                *current_path = child_path;
                self.current_chunk.replace(with_init_luau);
            }
            return Ok(())
        } else {
            let mut child_path_with_ext = child_path.clone();
            child_path_with_ext.set_extension("luau");
            if child_path_with_ext.is_file() {
                *current_path = child_path;
                self.current_chunk.replace(child_path_with_ext);
                return Ok(())
            }
        };
        Err(LuaNavigateError::NotFound)
    }
    fn is_module_present(&self) -> bool {
        self.current_chunk.borrow().is_file()
    }
    fn contents(&self) -> std::io::Result<Vec<u8>> {
        let current_chunk = self.current_chunk.borrow().clone();
        fs::read(&current_chunk)
    }
    fn chunk_name(&self) -> String {
        self.current_chunk.borrow().display().to_string()
    }
    fn cache_key(&self) -> Vec<u8> {
        self.current_chunk.borrow().to_string_lossy().as_bytes().to_vec()
    }
}
