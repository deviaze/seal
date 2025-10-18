use std::path::{Path, PathBuf};

use mluau::prelude::*;
use crate::prelude::*;
use crate::std_err::ecall;
use crate::{require, std_io};

pub fn error(_luau: &Lua, error_value: LuaValue) -> LuaValueResult {
    wrap_err!("message: {:?}", error_value.to_string()?)
}

pub fn warn(luau: &Lua, warn_value: LuaValue) -> LuaValueResult {
    let formatted_text = std_io::format::pretty(luau, warn_value)?;
    eprintln!("{}[WARN]{} {}{}", colors::BOLD_YELLOW, colors::RESET, formatted_text, colors::RESET);
    Ok(LuaNil)
}

pub const SEAL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn set_globals<S: AsRef<str>>(luau: &Lua, entry_path: S) -> LuaValueResult {
    let globals: LuaTable = luau.globals();
    // must use globals().get instead of globals().raw_get due to safeenv/sandbox (which requires newindex); raw_get incorrectly returns nil when safeenv enabled
    let luau_version: LuaString = globals.get("_VERSION")?;
    globals.raw_set("require", luau.create_function(require::require)?)?;
    globals.raw_set("error", luau.create_function(error)?)?;	
    globals.raw_set("p", luau.create_function(std_io::output::simple_print_and_return)?)?;
    globals.raw_set("pp", luau.create_function(std_io::output::pretty_print_and_return)?)?;
    globals.raw_set("dp", luau.create_function(std_io::output::debug_print)?)?;
    globals.raw_set("print", luau.create_function(std_io::output::pretty_print)?)?;
    globals.raw_set("ecall", luau.create_function(ecall)?)?;
    globals.raw_set("warn", luau.create_function(warn)?)?;
    globals.raw_set("_SEAL_VERSION", SEAL_VERSION)?;
    globals.raw_set("_VERSION", format!("seal {} | {}", SEAL_VERSION, luau_version.to_string_lossy()))?;
    globals.raw_set("_G", TableBuilder::create(luau)?.build()?)?;
    globals.raw_set("_REQUIRE_CACHE", TableBuilder::create(luau)?.build()?)?;
    globals.raw_set("script", TableBuilder::create(luau)?
        .with_value("entry_path", entry_path.as_ref())?
        .with_function("path", get_script_path)?
        .with_function("parent", get_script_parent)?
        .with_function("project", get_script_project)?
        .build_readonly()?
    )?;

    Ok(LuaNil)
}

const SCRIPT_PATH_SRC: &str = r#"
    requiring_file = ""
    local debug_name: string = (debug :: any).info(3, "s") --[[ this should give us the 
        debug name (set by luau.load().set_name) for the chunk that called require(),
        in the format `[string "./src/somewhere.luau"]`
    ]]
    requiring_file = string.sub(debug_name, 10, -3) -- grabs the part between `[string "` and `"]`
    return requiring_file
"#;

pub fn get_debug_name(luau: &Lua) -> LuaResult<String> {
    luau.load(SCRIPT_PATH_SRC).eval::<String>()
}

pub fn get_script_path(luau: &Lua, _multivalue: LuaMultiValue) -> LuaValueResult {
    let requiring_file = get_debug_name(luau)?;
    let requiring_file = luau.create_string(&requiring_file)?;
    Ok(LuaValue::String(requiring_file))
}

pub fn get_script_parent(luau: &Lua, _multivalue: LuaMultiValue) -> LuaValueResult {
    let requiring_parent = {
        let result: LuaString = luau.load(SCRIPT_PATH_SRC).eval()?;
        let script_path = result.to_string_lossy();
        match std::path::PathBuf::from(script_path).parent() {
            Some(parent) => parent.to_string_lossy().to_string(),
            None => {
                return wrap_err!("script:parent(): script does not have a parent :skull 💀:");
            }
        }
    };
    let parent_string = luau.create_string(&requiring_parent)?;
    Ok(LuaValue::String(parent_string))
}

pub fn find_project(path: &str, projects_up: usize) -> Option<PathBuf> {
    let mut current = Path::new(path).to_path_buf();

    if current.is_file() {
        current = current.parent()?.to_path_buf();
    }

    let mut matches = 0;

    loop {
        let seal_dir = current.join(".seal");
        if seal_dir.is_dir() {
            matches += 1;
            if matches == projects_up {
                return Some(current);
            }
        }

        // Move up one directory
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break, // Reached filesystem root
        }
    }

    None
}

pub fn get_script_project(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "script:project(n: number?)";
    pop_self(&mut multivalue, function_name)?;
    let projects_up = match multivalue.pop_front() {
        Some(LuaValue::Integer(i)) => int_to_usize(i, function_name, "n")?,
        Some(LuaValue::Number(f)) => float_to_usize(f, function_name, "n")?,
        Some(LuaNil) | None => 1,
        Some(other) => {
            return wrap_err!("{} expected number of projects up to be a number or nil/unspecified, got: {:?}", function_name, other);
        }
    };
    
    let requiring_file = get_debug_name(luau)?;
    match find_project(&requiring_file, projects_up) {
        Some(project) => ok_string(project.to_string_lossy().to_string(), luau),
        None => {
            wrap_err!("{}: project not found relative to '{}'! consider using fs.path.project instead (which doesn't error)", function_name, requiring_file)
        }
    }
}
