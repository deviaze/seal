use std::process::Command;
use std::io::{self, Write};

use crate::prelude::*;
use mluau::prelude::*;

use super::format;
use crate::std_err::WrappedError;

pub fn debug_print(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaResult<LuaString> {
    let function_name = "dp(...: any)";
    let mut result = String::from("");

    while let Some(value) = multivalue.pop_front() {
        format::process_debug_values(value, &mut result, 0)?;
        if !multivalue.is_empty() {
            result.push_str(", ");
        }
    }

    let debug_info = DebugInfo::from_caller(luau, function_name)?;
    println!(
        "{}[DEBUG]{} {}:{} in {}{}\n{}", 
        colors::BOLD_RED, colors::RESET, debug_info.source.replace("string ", ""), debug_info.line, debug_info.function_name, colors::RESET, 
        &result
    );
    luau.create_string(&result)
}

const OUTPUT_FORMATTER_SRC: &str = include_str!("./output_formatter.luau");

pub fn simple_print_and_return(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let r: LuaTable = luau.load(OUTPUT_FORMATTER_SRC).eval()?;
    let format_simple: LuaFunction = r.raw_get("simple")?;
    let mut result = String::from("");
    
    while let Some(value) = multivalue.pop_front() {
        match format_simple.call::<LuaString>(value) {
            Ok(text) => {
                let text = text.to_string_lossy();
                result += &text;
            },
            Err(err) => {
                return wrap_err!("p: error printing: {}", err);
            }
        };
        if !multivalue.is_empty() {
            result += ", ";
        }
    }

    println!("{}", &result);
    let result = luau.create_string(&result)?;
    Ok(LuaValue::String(result))
}

pub fn pretty_print(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaResult<()> {
    let r: LuaTable = luau.load(OUTPUT_FORMATTER_SRC).eval()?;
    let format_pretty: LuaFunction = r.raw_get("pretty")?;
    let mut result = String::from("");

    while let Some(value) = multivalue.pop_front() {
        match format_pretty.call::<LuaString>(value) {
            Ok(text) => {
                let text = text.to_string_lossy();
                result += &text;
            },
            Err(err) => {
                return wrap_err!("print: error printing: {}", err);
            }
        };
        if !multivalue.is_empty() {
            result += ", ";
        }
    }
    println!("{}", &result);
    Ok(())
}

pub fn pretty_print_and_return(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaResult<String> {
    let r: LuaTable = luau.load(OUTPUT_FORMATTER_SRC).eval()?;
    let format_pretty: LuaFunction = r.raw_get("pretty")?;
    let mut result = String::from("");

    while let Some(value) = multivalue.pop_front() {
        match format_pretty.call::<LuaString>(value) {
            Ok(text) => {
                let text = text.to_string_lossy();
                result += &text;
            },
            Err(err) => {
                return wrap_err!("pp: error printing: {}", err);
            }
        };
        if !multivalue.is_empty() {
            result += ", ";
        }
    }
    println!("{}", &result);
    Ok(result)
}

pub fn clear(_luau: &Lua, _value: LuaValue) -> LuaValueResult {
    let mut clear_command = if cfg!(target_os = "windows") {
        // use "cmd.exe /C cls" for Windows
        let mut com = Command::new("cmd");
        com.args(["/C", "cls"]);
        com
    } else {
        // use "clear" for Unix-like systems
        Command::new("clear")
    };
    match clear_command.spawn() {
        Ok(_) => {
            // this is pretty cursed, but yields long enough for the clear to have been completed 
            // otherwise the next print() calls get erased
            std::thread::sleep(std::time::Duration::from_millis(20));
            Ok(LuaNil)
        },
        Err(err) => {
            wrap_err!("output.clear: unable to clear the terminal: {}", err)
        }
    }
}

pub fn output_write(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "output.write(contents: string | buffer)";
    let contents = match value {
        LuaValue::String(text) => {
            text.as_bytes().to_owned()
        },
        LuaValue::Buffer(buffy) => {
            buffy.to_vec()
        },
        LuaNil => {
            return wrap_err!("{} expected contents to be a string or buffer, got nothing or nil", function_name);
        },
        other => {
            return wrap_err!("{} expected contents to be a string or buffer, got {:?}", function_name, other);
        }
    };

    // we can't give users explicit flush control because stdout/stderr flushes anyway
    // when it goes out of scope and it's not worth it to keep stdout/stderr around for multiple calls
    let should_flush = !contents.ends_with(b"\n");

    let mut stdout = io::stdout();
    if let Err(err) = stdout.write_all(&contents) {
        return WrappedError::from_message(err.to_string()).get_userdata(luau);
    }

    if should_flush && let Err(err) = stdout.flush() {
        return WrappedError::from_message(err.to_string()).get_userdata(luau);
    }

    Ok(LuaNil)
}

pub fn output_ewrite(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "output.ewrite(contents: string | buffer)";
    let contents = match value {
        LuaValue::String(text) => {
            text.as_bytes().to_owned()
        },
        LuaValue::Buffer(buffy) => {
            buffy.to_vec()
        },
        LuaNil => {
            return wrap_err!("{} expected contents to be a string or buffer, got nothing or nil", function_name);
        },
        other => {
            return wrap_err!("{} expected contents to be a string or buffer, got {:?}", function_name, other);
        }
    };

    // we can't give users explicit flush control because stdout/stderr flushes anyway
    // when it goes out of scope and it's not worth it to keep stdout/stderr around for multiple calls
    let should_flush = !contents.ends_with(b"\n");

    let mut stderr = io::stderr();
    if let Err(err) = stderr.write_all(&contents) {
        return WrappedError::from_message(err.to_string()).get_userdata(luau);
    }

    if should_flush && let Err(err) = stderr.flush() {
        return WrappedError::from_message(err.to_string()).get_userdata(luau);
    }

    Ok(LuaNil)
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("clear", clear)?
        .with_function("write", output_write)?
        .with_function("ewrite", output_ewrite)?
        .with_function("sprint", simple_print_and_return)?
        .build_readonly()
}
