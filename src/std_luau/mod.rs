use mluau::prelude::*;
use crate::prelude::*;

use mluau::Compiler;

// TODO: lute support

struct EvalError {
    message: String,
}
impl EvalError {
    fn new(err: LuaError) -> Self {
        Self {
            message: err.to_string()
        }
    }
}
impl LuaUserData for EvalError {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_meta_field("__type", "error"); // allow users to typeof check
    }
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, | luau: &Lua, this: &EvalError, _: LuaValue| -> LuaValueResult {
            this.message.clone().into_lua(luau)
        });
    }
}

enum EvalStdlib {
    Seal,
    Safe,
    None,
}

struct EvalOptions {
    name: Option<String>,
    stdlib: EvalStdlib,
}
impl EvalOptions {
    fn default() -> Self {
        EvalOptions {
            name: None,
            stdlib: EvalStdlib::Safe,
        }
    }

    fn from_value(value: LuaValue, function_name: &'static str) -> LuaResult<Self> {
        let t = match value {
            LuaValue::Table(t) => Some(t),
            LuaNil => None,
            other => {
                return wrap_err!("{} expected EvalOptions to be a table (with fields stdlib and/or is_bytecode) or nil, got: {:?}", function_name, other);
            }
        };
        if t.is_none() {
            return Ok(Self::default());
        }

        // SAFETY: we just checked .is_none()
        let t = unsafe { t.unwrap_unchecked() };

        let stdlib = match t.raw_get("stdlib")? {
            LuaValue::String(s) => {
                let s_bytes = s.as_bytes();
                if s_bytes == &b"seal"[..] {
                    EvalStdlib::Seal
                } else if s_bytes == &b"safe"[..] {
                    EvalStdlib::Safe
                } else if s_bytes == &b"none"[..] {
                    EvalStdlib::None
                } else {
                    return wrap_err!("{} expected EvalOptions.stdlib to be \"seal\" or \"safe\" or \"none\" or nil, got an invalid string: {}", function_name, s.display())
                }
            },
            LuaNil => EvalStdlib::Safe,
            other => {
                return wrap_err!("{} expected EvalOptions.stdlib to be \"seal\" or \"safe\" or \"none\" or nil, got: {:?}", function_name, other);
            }
        };

        let name = match t.raw_get("name")? {
            Some(LuaValue::String(name)) => {
                Some(name.to_string_lossy())
            },
            Some(LuaNil) | None => {
                None
            },
            Some(other) => {
                return wrap_err!("{} expected name to be a string or nil, got: {:?}", function_name, other);
            }
        };

        Ok(EvalOptions {
            name,
            stdlib,
        })
    }
}

fn get_safe_globals(luau: &Lua) -> LuaResult<LuaTable> {
    let safe_globals = [
        // luau standard libraries
        "math", "table", "string", "coroutine", "bit32", "utf8", "os", "debug", "buffer", "vector",
        // some useful functions
        "assert", "error", "getmetatable", "setmetatable", "next", "ipairs", "pairs", "rawequal", "rawget", "rawset", "setmetatable", 
        "tonumber", "tostring", "type", "typeof", "pcall", "xpcall", "unpack", "print"
        // note that require is purposely not included
    ];
    let t = luau.create_table()?;
    for glob in safe_globals {
        let value = luau.globals().get::<LuaValue>(glob)?;
        t.raw_set(glob, value)?;
    };
    t.raw_set("_VERSION", "Luau")?;
    let dummy_require_fn = luau.create_function(|_l: &Lua, _v: LuaValue| -> LuaValueResult {
        wrap_err!("require is not allowed here!")
    })?;
    t.raw_set("require", dummy_require_fn)?;
    Ok(t)
}

/// this function is unsafe because invalid bytecode can cause the interpreter to crash and seal to crash
/// with illegal instruction & core dump. caller is responsible for making sure valid bytecode is passed
unsafe fn eval(luau: &Lua, src: Vec<u8>, eval_options: EvalOptions) -> LuaValueResult {
    let name = eval_options.name.unwrap_or("luau.load".to_string());
    let chunk = match eval_options.stdlib {
        EvalStdlib::Safe => {
            luau.load(src).set_name(name).set_environment(get_safe_globals(luau)?)
        },
        EvalStdlib::None => {
            luau.load(src).set_name(name).set_environment(luau.create_table()?)
        },
        EvalStdlib::Seal => {
            luau.load(src).set_name(name)
        }
    };
    let res = match chunk.eval::<LuaValue>() {
        Ok(value) => value,
        Err(err) => {
            LuaValue::UserData(luau.create_userdata(EvalError::new(err))?)
        }
    };

    Ok(res)
}

fn luau_eval(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "luau.eval(src: string, options: EvalOptions?)";
    let src = match multivalue.pop_front() {
        Some(LuaValue::String(src)) => {
            if let Ok(src) = src.to_str() {
                src.as_bytes().to_owned()
            } else {
                return wrap_err!(
                    "{}: Your src appears to be bytecode!\n\nEvaluating bytecode is UNSAFE because it can cause the entire program to crash!! (with an illegal instruction coredump!!!)!\n\nIf you trust your bytecode is valid and safe, use luau.eval_unsafe to evaluate it \n(please don't blame me if seal crashes).",
                    function_name
                )
            }
        },
        Some(LuaNil) | None => {
            return wrap_err!("{} expected src to be a string of Luau source code, but was incorrectly called with zero arguments", function_name);
        },
        other => {
            return wrap_err!("{} expected src to be a string of Luau source code, got: {:?}", function_name, other);
        }
    };
    let eval_options = match multivalue.pop_front() {
        Some(v) => EvalOptions::from_value(v, function_name)?,
        None => EvalOptions::default(),
    };

    // SAFETY: we've verified src is not luau bytecode (which must be invalid utf-8)
    let res = unsafe { eval(luau, src, eval_options) }?;
    Ok(res)
}

// this function is actually unsafe but we can't pass unsafe functions to luau
fn luau_eval_unsafe(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "luau.eval(src: string, options: EvalOptions?)";
    let src = match multivalue.pop_front() {
        Some(LuaValue::String(src)) => src.as_bytes().to_owned(),
        Some(LuaValue::Buffer(buffy))=> buffy.to_vec(),
        Some(LuaNil) | None => {
            return wrap_err!("{} expected src to be a string (of either source code or bytecode) or buffer, but was incorrectly called with zero arguments", function_name);
        },
        other => {
            return wrap_err!("{} expected src to be a string (of either source code or bytecode) or buffer, got: {:?}", function_name, other);
        }
    };
    let eval_options = match multivalue.pop_front() {
        Some(v) => EvalOptions::from_value(v, function_name)?,
        None => EvalOptions::default(),
    };

    // SAFETY: caller in Luau is responsible for passing valid bytecode
    // if/when invalid bytecode is passed to this function, seal will crash with an "illegal instruction" and coredump.
    let res = unsafe { eval(luau, src, eval_options) }?;
    Ok(res)
}

fn luau_bytecode(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "luau.bytecode(src: string)";
    let src = match value {
        LuaValue::String(src) => src.to_string_lossy(),
        other => {
            return wrap_err!("{} expected src to be a string, got: {:?}", function_name, other);
        }
    };
    let comp = Compiler::new();
    let res = match comp.compile(src) {
        Ok(bytecode) => bytecode,
        Err(err) => {
            return Ok(LuaValue::UserData(
                luau.create_userdata(EvalError::new(err))?
            ))
        }
    };
    ok_buffy(res, luau)
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("eval", luau_eval)?
        .with_function("eval_unsafe", luau_eval_unsafe)?
        .with_function("bytecode", luau_bytecode)?
        .build_readonly()
}