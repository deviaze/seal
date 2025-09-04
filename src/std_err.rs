use crate::prelude::*;
use mluau::prelude::*;
use crate::err;

pub struct WrappedError {
    message: String,
    traceback: Option<String>,
}

impl WrappedError {
    pub fn from_message(message: String) -> Self {
        Self {
            message,
            traceback: None
        }
    }
    pub fn with_traceback(message: String, luau: &Lua) -> LuaResult<Self> {
        let traceback = Some(err::parse_traceback(luau.traceback()?));
        Ok(Self {
            message,
            traceback,
        })
    }
    pub fn format(&self) -> String {
        let traceback = self.traceback.clone().unwrap_or_default();
        if traceback.is_empty() {
            // format!("{}[ERR]{} {}", colors::BOLD_RED, colors::RESET, self.message)
            self.message.clone()
        } else {
            format!("{}{}{}\n{}\n", colors::RED, self.message, colors::RESET, traceback)
            // format!("{}[ERR]{} {}\n{}", colors::BOLD_RED, colors::RESET, self.message, traceback)
        }
    }
    pub fn get_userdata(self, luau: &Lua) -> LuaValueResult {
        ok_userdata(self, luau)
    }
}

impl LuaUserData for WrappedError {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_meta_field("__type", "error"); // allow users to typeof check
    }
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, | luau: &Lua, this: &WrappedError, _: LuaValue| -> LuaValueResult {
            this.message.clone().into_lua(luau)
        });
    }
}

fn err_message(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "err.message(m: string)";
    let message = match value {
        LuaValue::String(message) => message.to_string_lossy(),
        other => {
            return wrap_err!("{}: error message must be a string, got: {:?}", function_name, other);
        }
    };
    WrappedError::from_message(message).get_userdata(luau)
}

fn err_wrap(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "err.wrap(message: string)";
    let message = match value {
        LuaValue::String(message) => {
            format!("{}{}{}", colors::RED, message.to_string_lossy(), colors::RESET)
        },
        other => {
            return wrap_err!("{}: error message must be a string, got: {:?}", function_name, other);
        }
    };
    WrappedError::with_traceback(message, luau)?.get_userdata(luau)
}

fn format_error(value: LuaValue) -> LuaResult<String> {
    let stringified = match value {
        LuaValue::UserData(ud) => {
            if let Ok(wrapped) = ud.borrow::<WrappedError>() {
                wrapped.format()
            } else {
                let err = err::parse_traceback(ud.to_string()?);
                format!("{}[ERR]{} {}", colors::BOLD_RED, colors::RESET, err)
            }
        },
        LuaValue::Error(err) => {
            err::parse_traceback(err.to_string())
            // format!("{}[ERR]{} {}", colors::BOLD_RED, colors::RESET, err)
        }
        other => {
            return wrap_err!("passed error isn't the expected userdata, got: {:?}", other);
        }
    };
    Ok(stringified)
}

fn err_format(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let formatted = format_error(value)?;
    ok_string(formatted, luau)
}

fn err_traceback(luau: &Lua, _: LuaValue) -> LuaValueResult {
    let traceback = luau.traceback()?;
    ok_string(err::parse_traceback(traceback), luau)
}

fn err_throw(_luau: &Lua, value: LuaValue) -> LuaValueResult {
    let formatted = format_error(value)?;
    Err(LuaError::external(formatted))
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("message", err_message)?
        .with_function("wrap", err_wrap)?
        .with_function("format", err_format)?
        .with_function("traceback", err_traceback)?
        .with_function("throw", err_throw)?
        .build_readonly()
}