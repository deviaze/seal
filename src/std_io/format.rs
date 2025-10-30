use crate::prelude::*;
use mluau::prelude::*;

use regex::Regex;

pub fn process_debug_values(value: LuaValue, result: &mut String, depth: usize) -> LuaResult<()> {
    let left_padding = " ".repeat(2 * depth);
    match value {
        LuaValue::Table(t) => {
            if depth < 10 {
                result.push_str("{\n");
                for pair in t.pairs::<LuaValue, LuaValue>() {
                    let (k, v) = pair?;
                    result.push_str(&format!("  {left_padding}{:#?} = ", k));
                    process_debug_values(v, result, depth + 1)?;
                    result.push('\n');
                }
                result.push_str(&format!("{left_padding}}}"));
            }
        },
        LuaValue::String(s) => {
            let formatted_string = format!("{:?}", s);
            result.push_str(&formatted_string);
        },
        LuaValue::UserData(data) => {
            match data.call_method::<LuaString>("__dp", ()) {
                Ok(dp_output) => {
                    result.push_str(&dp_output.to_string_lossy());
                },
                Err(_) => {
                    // __dp isn't defined or fails
                    result.push_str(&format!("{:?}", data));
                }
            }
        },
        _ => {
            result.push_str(&format!("{:?}", value));
        }
    }
    if depth > 0 {
        result.push(',');
    }
    Ok(())
}

fn debug(luau: &Lua, stuff: LuaMultiValue) -> LuaResult<LuaString> {
    let mut result = String::from("");
    let mut multi_values = stuff.clone();

    while let Some(value) = multi_values.pop_front() {
        process_debug_values(value, &mut result, 0)?;
        if !multi_values.is_empty() {
            result += ", ";
        }
    }

    luau.create_string(&result)
}

const OUTPUT_FORMATTER_SRC: &str = include_str!("./output_formatter.luau");
pub fn simple(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let r: LuaTable = luau.load(OUTPUT_FORMATTER_SRC).eval()?;
    let format_simple: LuaFunction = r.raw_get("simple")?;
    let result = match format_simple.call::<LuaString>(value) {
        Ok(text) => text.to_string_lossy(),
        Err(err) => {
            return wrap_err!("format.simple: error formatting: {}", err);
        }
    };

    let result = luau.create_string(&result)?;
    Ok(LuaValue::String(result))
}

pub fn strip_colors(input: &str) -> String {
    let re_colors = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    let without_colors = re_colors.replace_all(input, "");
    without_colors.to_string()
}

fn uncolor(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let input = match value {
        LuaValue::String(s) => s.to_string_lossy(),
        other => {
            return wrap_err!("expected string to strip formatting of, got: {:#?}", other)
        }
    };
    let input = strip_colors(&input);
    Ok(LuaValue::String(
        luau.create_string(input.as_str())?
    ))
}

pub fn pretty(luau: &Lua, value: LuaValue) -> LuaResult<String> {
    let r: LuaTable = luau.load(OUTPUT_FORMATTER_SRC).eval()?;
    let format_pretty: LuaFunction = r.raw_get("pretty")?;
    let result = match format_pretty.call::<LuaString>(value) {
        Ok(text) => text.to_string_lossy(),
        Err(err) => {
            return wrap_err!("format: error formatting: {}", err);
        }
    };
    Ok(result)
}

pub fn __call_format(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaResult<String> {
    let function_name = "io.format(item: unknown)";
    pop_self(&mut multivalue, function_name)?;
    let value = match multivalue.pop_front() {
        Some(value) => value,
        None => {
            return wrap_err!("{} expected an item to format, got nothing at all (not even nil)", function_name);
        }
    };
    pretty(luau, value)
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("pretty", pretty)?
        .with_function("simple", simple)?
        .with_function("debug", debug)?
        .with_function("uncolor", uncolor)?
        .with_metatable(TableBuilder::create(luau)?
            .with_function("__call", __call_format)?
            .build_readonly()?
        )?
        .build_readonly()
}