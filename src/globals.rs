use mlua::prelude::*;
use crate::prelude::*;
use crate::{require, std_io};

pub fn error(_luau: &Lua, error_value: LuaValue) -> LuaValueResult {
    wrap_err!("message: {:?}", error_value.to_string()?)
}

pub fn warn(luau: &Lua, warn_value: LuaValue) -> LuaValueResult {
    let formatted_text = std_io::output::format_output(luau, warn_value)?;
    println!("{}{}{}", colors::BOLD_YELLOW, formatted_text, colors::RESET);
    Ok(LuaNil)
}

const SEAL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn set_globals(luau: &Lua) -> LuaValueResult {
    let globals: LuaTable = luau.globals();
    let luau_version: LuaString = globals.raw_get("_VERSION")?;
    globals.raw_set("require", luau.create_function(require::require)?)?;
    globals.raw_set("error", luau.create_function(error)?)?;	
    globals.raw_set("p", luau.create_function(std_io::output::simple_print_and_return)?)?;
    globals.raw_set("pp", luau.create_function(std_io::output::pretty_print_and_return)?)?;
    globals.raw_set("dp", luau.create_function(std_io::output::debug_print)?)?;
    globals.raw_set("print", luau.create_function(std_io::output::pretty_print)?)?;
    globals.raw_set("warn", luau.create_function(warn)?)?;
    globals.raw_set("_VERSION", format!("seal {} | {}", SEAL_VERSION, luau_version.to_string_lossy()))?;
    globals.raw_set("_G", TableBuilder::create(luau)?
        // .with_metatable(TableBuilder::create(luau)?
        //     .with_value("__index", luau.globals())?
        //     .build_readonly()?
        // )?
        .build()?
    )?;
    globals.raw_set("_REQUIRE_CACHE", TableBuilder::create(luau)?.build()?)?;

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

pub fn get_script_path(luau: &Lua, _multivalue: LuaMultiValue) -> LuaValueResult {
    let requiring_file = {
        let result: LuaString = luau.load(SCRIPT_PATH_SRC).eval()?;
        result.to_string_lossy()
    };
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
                return wrap_err!("script:path(): script does not have a parent");
            }
        }
    };
    let parent_string = luau.create_string(&requiring_parent)?;
    Ok(LuaValue::String(parent_string))
}