#![allow(clippy::single_char_add_str)]

use std::process::Command;
use std::io::{self, Write};

use regex::Regex;
use mluau::prelude::*;

use crate::prelude::*;

fn process_raw_values(value: LuaValue, result: &mut String, depth: usize) -> LuaResult<()> {
    let left_padding = " ".repeat(2 * depth);
    match value {
        LuaValue::Table(t) => {
            if depth < 10 {
                result.push_str("{\n");
                for pair in t.pairs::<LuaValue, LuaValue>() {
                    let (k, v) = pair?;
                    result.push_str(&format!("  {left_padding}{:?} = ", k));
                    process_raw_values(v, result, depth + 1)?;
                    result.push_str("\n");
                }
                result.push_str(&format!("{left_padding}}}"));
            }
        },
        LuaValue::String(s) => {
            let formatted_string = format!("String({:?})", s);
            result.push_str(&formatted_string);
        },
        _ => {
            result.push_str(&format!("{:?}", value));
        }
    }
    if depth > 0 {
        result.push_str(",");
    }
    Ok(())
}

pub fn debug_print(luau: &Lua, stuff: LuaMultiValue) -> LuaResult<LuaString> {
    let mut result = String::from("");
    let mut multi_values = stuff.clone();

    while let Some(value) = multi_values.pop_front() {
        process_raw_values(value, &mut result, 0)?;
        if !multi_values.is_empty() {
            result += ", ";
        }
    }

    println!("{:?}", result.clone());
    luau.create_string(&result)
}

fn format_debug(luau: &Lua, stuff: LuaMultiValue) -> LuaResult<LuaString> {
    let mut result = String::from("");
    let mut multi_values = stuff.clone();

    while let Some(value) = multi_values.pop_front() {
        process_raw_values(value, &mut result, 0)?;
        if !multi_values.is_empty() {
            result += ", ";
        }
    }

    luau.create_string(&result)
}

const OUTPUT_PROCESS_VALUES: &str = include_str!("./output_formatter.luau");

pub fn simple_print_and_return(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let r: LuaTable = luau.load(OUTPUT_PROCESS_VALUES).eval()?;
    let format_simple: LuaFunction = r.raw_get("simple_print")?;
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

pub fn simple_format(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let r: LuaTable = luau.load(OUTPUT_PROCESS_VALUES).eval()?;
    let format_simple: LuaFunction = r.raw_get("simple_print")?;
    let result = match format_simple.call::<LuaString>(value) {
        Ok(text) => text.to_string_lossy(),
        Err(err) => {
            return wrap_err!("sformat: error formatting: {}", err);
        }
    };

    let result = luau.create_string(&result)?;
    Ok(LuaValue::String(result))
}

pub fn pretty_print(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaResult<()> {
    let r: LuaTable = luau.load(OUTPUT_PROCESS_VALUES).eval()?;
    let format_pretty: LuaFunction = r.raw_get("pretty_print")?;
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
    let r: LuaTable = luau.load(OUTPUT_PROCESS_VALUES).eval()?;
    let format_pretty: LuaFunction = r.raw_get("pretty_print")?;
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

pub fn format_output(luau: &Lua, value: LuaValue) -> LuaResult<String> {
    let r: LuaTable = luau.load(OUTPUT_PROCESS_VALUES).eval()?;
    let format_pretty: LuaFunction = r.raw_get("pretty_print")?;
    let result = match format_pretty.call::<LuaString>(value) {
        Ok(text) => text.to_string_lossy(),
        Err(err) => {
            return wrap_err!("format: error formatting: {}", err);
        }
    };
    Ok(result)
}

pub fn format(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let formatted = format_output(luau, value)?;
    Ok(LuaValue::String(luau.create_string(formatted)?))
}

pub fn strip_newlines_and_colors(input: &str) -> String {
    let re_colors = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    let without_colors = re_colors.replace_all(input, "");
    without_colors.to_string()
}

fn output_unformat(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let input = match value {
        LuaValue::String(s) => s.to_string_lossy(),
        other => {
            return wrap_err!("expected string to strip formatting of, got: {:#?}", other)
        }
    };
    let input = strip_newlines_and_colors(&input);
    Ok(LuaValue::String(
        luau.create_string(input.as_str())?
    ))
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

pub fn output_write(_luau: &Lua, value: LuaValue) -> LuaValueResult {
    match value {
        LuaValue::String(text) => {
            io::stdout().write_all(text.to_string_lossy().as_bytes()).unwrap();
            io::stdout().flush().unwrap();
            Ok(LuaNil)
        },
        LuaValue::Buffer(buffy) => {
            io::stdout().write_all(&buffy.to_vec()).unwrap();
            io::stdout().flush().unwrap();
            Ok(LuaNil)
        }
        other => {
            wrap_err!("io.output.write: expected string or buffer, got: {:#?}", other)
        }
    }
}

pub fn output_ewrite(_luau: &Lua, value: LuaValue) -> LuaValueResult {
    match value {
        LuaValue::String(text) => {
            io::stderr().write_all(&text.as_bytes()).unwrap();
            io::stderr().flush().unwrap();
            Ok(LuaNil)
        },
        LuaValue::Buffer(buffy) => {
            io::stderr().write_all(&buffy.to_vec()).unwrap();
            io::stderr().flush().unwrap();
            Ok(LuaNil)
        }
        other => {
            wrap_err!("io.output.ewrite: expected string or buffer, got: {:#?}", other)
        }
    }
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("format", format)?
        .with_function("clear", clear)?
        .with_function("write", output_write)?
        .with_function("ewrite", output_ewrite)?
        .with_function("unformat", output_unformat)?
        .with_function("sprint", simple_print_and_return)?
        .with_function("sformat", simple_format)?
        .with_function("print-and-return", pretty_print_and_return)?
        .with_function("debug-print", debug_print)?
        .with_function("debug-format", format_debug)?
        .build_readonly()
}
