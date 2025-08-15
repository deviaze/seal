use std::collections::VecDeque;
use std::ffi::OsString;

use mluau::prelude::*;
use crate::prelude::*;
use crate::{globals, std_env};
use crate::std_fs::validate_path;

use include_dir::{Dir, include_dir};

const SETUP_SRC: &str = include_str!("./setup.luau");

const TYPEDEFS_DIR: Dir = include_dir!("./.seal/typedefs");
const EXTRA_DIR: Dir = include_dir!("./.seal/extra");
const GUIDED_TOUR_SRC: &str = include_str!("../../.seal/guided_tour.luau");
const DEFAULT_CONFIG_SRC: &str = include_str!("./default_config.luau");

fn extract_typedefs(_luau: &Lua, value: LuaValue) -> LuaEmptyResult {
    let function_name = "seal setup - extract_typedefs";
    let path = match value {
        LuaValue::String(path) => validate_path(&path, function_name)?,
        other => {
            return wrap_err!("{}: expected path to be a string, got: {:?}", function_name, other);
        }
    };
    match TYPEDEFS_DIR.extract(&path) {
        Ok(_) => Ok(()),
        Err(err) => {
            wrap_err!("{}: unable to extract typedefs directory to path '{}' due to err: {}", function_name, path, err)
        }
    }
}

fn extract_extra(_luau: &Lua, value: LuaValue) -> LuaEmptyResult {
    let function_name = "seal setup - extract_extra";
    let path = match value {
        LuaValue::String(path) => validate_path(&path, function_name)?,
        other => {
            return wrap_err!("{}: expected path to be a string, got: {:?}", function_name, other);
        }
    };
    match EXTRA_DIR.extract(&path) {
        Ok(_) => Ok(()),
        Err(err) => {
            wrap_err!("{}: unable to extract @extra directory to path '{}' due to err: {}", function_name, path, err)
        }
    }
}

pub fn create_internal(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("extract_typedefs", extract_typedefs)?
        .with_function("extract_extra", extract_extra)?
        .with_value("GUIDED_TOUR_SRC", GUIDED_TOUR_SRC.into_lua(luau)?)?
        .with_value("DEFAULT_CONFIG_SRC", DEFAULT_CONFIG_SRC.into_lua(luau)?)?
        .build_readonly()
}

#[derive(Debug)]
pub enum SetupOptions {
    Default,
    Project,
    Script,
    Custom,
}

impl SetupOptions {
    pub fn from_args(args: &VecDeque<OsString>) -> LuaResult<Self> {
        Ok(if let Some(front) = args.front() {
            if front == &OsString::from("project") {
                SetupOptions::Project
            } else if front == &OsString::from("script") {
                SetupOptions::Script
            } else if front == &OsString::from("custom") {
                SetupOptions::Custom
            } else {
                return wrap_err!("unexpected seal setup option: {}", front.display());
            }
        } else {
            SetupOptions::Default
        })
    }
}

pub fn run(options: SetupOptions) -> LuaEmptyResult {
    let cwd = std_env::get_cwd("seal setup")?;
    let temp_luau = Lua::default();
    globals::set_globals(&temp_luau, cwd.to_string_lossy())?;
    let setup_table = match temp_luau.load(SETUP_SRC).set_name("seal setup").eval::<LuaValue>() {
        Ok(t) => match t {
            LuaValue::Table(t) => t,
            other => {
                panic!("seal's setup.luau unexpectedly returned a {:?}; expected table", other);
            }
        },
        Err(err) => {
            return wrap_err!("seal setup.luau errored at runtime: {}", err);
        }
    };
    let defaults_table = match setup_table.raw_get("defaults")? {
        LuaValue::Table(t) => t,
        other => panic!("defaults table not a table? got: {:?}", other),
    };
    let setup_function = match setup_table.raw_get::<LuaValue>("setup")? {
        LuaValue::Function(f) => f,
        other => panic!("seal setup.luau's setup function not a function? got: {:?}", other),
    };
    setup_function.call::<()>(match options {
        SetupOptions::Default => defaults_table.raw_get("default")?,
        SetupOptions::Project => defaults_table.raw_get("project")?,
        SetupOptions::Script => defaults_table.raw_get("script")?,
        SetupOptions::Custom => LuaNil,
    })
}