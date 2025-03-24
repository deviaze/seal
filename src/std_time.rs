use crate::{table_helpers::TableBuilder, wrap_err, colors};
use crate::LuaValueResult;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{thread, time::Duration};

use chrono::Local;
use mlua::prelude::*;

fn time_wait(_luau: &Lua, seconds: LuaNumber) -> LuaValueResult {
    let millis = (seconds * 1000.0) as u64;
    let dur = Duration::from_millis(millis);
    thread::sleep(dur);
    Ok(LuaValue::Boolean(true)) // return true to ensure while time.wait(n) works
}

pub fn from_system_time(luau: &Lua, system_time: SystemTime) -> LuaValueResult {
    let unix_timestamp = system_time.duration_since(UNIX_EPOCH)
        .expect("time went oof");
    let unix_timestamp = unix_timestamp.as_secs();
    time_datetime_from(luau, unix_timestamp.into_lua(luau)?)
}

pub fn time_datetime_from(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let unix_timestamp = match value { 
        LuaValue::Integer(unix_timestamp) => unix_timestamp,
        LuaValue::Number(maybe_timestamp) => {
            maybe_timestamp as i32
        },
        other =>  {
            return wrap_err!("datetime.from(unix_timestamp) expected unix_timestamp to be a number, got: {:?}", other);
        }
    };

    let dt = chrono::DateTime::from_timestamp(unix_timestamp.into(), 0).unwrap();
    Ok(LuaValue::Table(
        TableBuilder::create(luau)?
            .with_value("unix_timestamp", dt.timestamp())?
            .with_function("format_utc", move |luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                match multivalue.pop_back() {
                    Some(LuaValue::String(format_string)) => {
                        let format_string = format_string.to_str()?.to_string();
                        dt.format(&format_string).to_string().into_lua(luau)
                    }, 
                    other => {
                        wrap_err!("DateTime.format expected format string to be a string, got: {:?}", other)
                    }
                }
            })?
            .with_function("format_local", move |luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                let local_dt = dt.with_timezone(&Local);
                match multivalue.pop_back() {
                    Some(LuaValue::String(format_string)) => {
                        let format_string = format_string.to_str()?.to_string();
                        local_dt.format(&format_string).to_string().into_lua(luau)
                    }, 
                    other => {
                        wrap_err!("DateTime.format expected format string to be a string, got: {:?}", other)
                    }
                }
            })?
            .build_readonly()?,
    ))
}

fn time_datetime_now(luau: &Lua, _: LuaValue) -> LuaValueResult {
    let now = chrono::Local::now();
    Ok(LuaValue::Table(
        TableBuilder::create(luau)?
            .with_value("unix_timestamp", now.timestamp())?
            .with_function("format", move |luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                match multivalue.pop_back() {
                    Some(LuaValue::String(format_string)) => {
                        let format_string = format_string.to_str()?.to_string();
                        now.format(&format_string).to_string().into_lua(luau)
                    }, 
                    other => {
                        wrap_err!("DateTime.format expected format string to be a string, got: {:?}", other)
                    }
                }
            })?
            .build_readonly()?,
    ))
}

pub fn create_datetime(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("now", time_datetime_now)?
        .with_function("from", time_datetime_from)?
        .with_value("common_formats", TableBuilder::create(luau)?
            .with_value("ISO_8601", "%Y-%m-%d %H:%M")?
            .with_value("RFC_2822", "%a, %d %b %Y %H:%M:%S %z")?
            .with_value("RFC_3339", "%Y-%m-%dT%H:%M:%S%:z")?
            .with_value("SHORT_DATE", "%Y-%m-%d")?
            .with_value("SHORT_TIME", "%H:%M")?
            .with_value("FULL_DATE_TIME", "%A, %B %d, %Y %H:%M:%S")?
            // Common American formats
            .with_value("MM/DD/YYYY", "%m/%d/%Y")?
            .with_value("MM/DD/YYYY HH:MM (AM/PM)", "%m/%d/%Y %I:%M %p")?
            .with_value("MM/DD/YY", "%m/%d/%y")?
            .with_value("HH:MM (AM/PM)", "%I:%M %p")?
            .with_value("AMERICAN_FULL_DATE_TIME", "%A, %B %d, %Y %I:%M:%S %p")?
            .build_readonly()?
        )?
        .build_readonly()
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("wait", time_wait)?
        .with_value("datetime", create_datetime(luau)?)?
        .build_readonly()
}
