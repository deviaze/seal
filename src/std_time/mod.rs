use crate::prelude::*;
use mluau::prelude::*;
use std::time::Duration;

pub mod datetime;
pub mod duration;

use duration::TimeSpan;
use duration::TimeDuration;

/// Sleeps for the given number of seconds using thread::sleep.
///
/// Precision is limited to the system's sleep granularity—typically around 1ms.
/// Values smaller than ~0.001 seconds may round down to zero and result in no actual delay.
/// For high-precision timing, consider alternatives such as spinning or avoid using very small floats.
///
/// This function is intended for simple time-based control in Luau.
/// Avoid using for profiling or real-time control.
fn time_wait(_luau: &Lua, seconds: LuaNumber) -> LuaValueResult {
    let millis = (seconds * 1000.0) as u64;
    let dur = Duration::from_millis(millis);
    std::thread::sleep(dur);
    Ok(LuaValue::Boolean(true)) // return true so while time.wait(1) loops still work
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("wait", time_wait)?
        .with_value("datetime", datetime::create(luau)?)?

        .with_function("days", |luau: &Lua, value: LuaValue| -> LuaValueResult {
            let function_name = "time.days(d: number)";
            let days = match value {
                LuaValue::Number(f) => f,
                LuaValue::Integer(i) => i as f64,
                other => {
                    return wrap_err!("{} expected number, got: {:?}", function_name, other);
                }
            };
            TimeDuration::days(days).get_userdata(luau)
        })?

        .with_function("hours", |luau: &Lua, value: LuaValue| -> LuaValueResult {
            let function_name = "time.hours(h: number)";
            let hours = match value {
                LuaValue::Number(f) => f,
                LuaValue::Integer(i) => i as f64,
                other => {
                    return wrap_err!("{} expected number, got: {:?}", function_name, other);
                }
            };
            TimeDuration::hours(hours).get_userdata(luau)
        })?

        .with_function("minutes", |luau: &Lua, value: LuaValue| -> LuaValueResult {
            let function_name = "time.minutes(m: number)";
            let minutes = match value {
                LuaValue::Number(f) => f,
                LuaValue::Integer(i) => i as f64,
                other => {
                    return wrap_err!("{} expected number, got: {:?}", function_name, other);
                }
            };
            TimeDuration::minutes(minutes).get_userdata(luau)
        })?

        .with_function("seconds", |luau: &Lua, value: LuaValue| -> LuaValueResult {
            let function_name = "time.seconds(s: number)";
            let seconds = match value {
                LuaValue::Number(f) => f,
                LuaValue::Integer(i) => i as f64,
                other => {
                    return wrap_err!("{} expected number, got: {:?}", function_name, other);
                }
            };
            TimeDuration::seconds(seconds).get_userdata(luau)
        })?

        .with_function("milliseconds", |luau: &Lua, value: LuaValue| -> LuaValueResult {
            let function_name = "time.milliseconds(ms: number)";
            let ms = match value {
                LuaValue::Number(f) => f,
                LuaValue::Integer(i) => i as f64,
                other => {
                    return wrap_err!("{} expected number, got: {:?}", function_name, other);
                }
            };
            TimeDuration::milliseconds(ms).get_userdata(luau)
        })?

        .with_function("microseconds", |luau: &Lua, value: LuaValue| -> LuaValueResult {
            let function_name = "time.microseconds(us: number)";
            let us = match value {
                LuaValue::Number(f) => f,
                LuaValue::Integer(i) => i as f64,
                other => {
                    return wrap_err!("{} expected number, got: {:?}", function_name, other);
                }
            };
            TimeDuration::microseconds(us).get_userdata(luau)
        })?

        .with_function("nanoseconds", |luau: &Lua, value: LuaValue| -> LuaValueResult {
            let function_name = "time.nanoseconds(ns: number)";
            let ns = match value {
                LuaValue::Number(f) => f,
                LuaValue::Integer(i) => i as f64,
                other => {
                    return wrap_err!("{} expected number, got: {:?}", function_name, other);
                }
            };
            TimeDuration::nanoseconds(ns).get_userdata(luau)
        })?

        .build_readonly()
}
