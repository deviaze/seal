use mlua::prelude::*;
use crate::globals;

pub fn parse_traceback(raw_traceback: String) -> String {
    let parse_traceback = include_str!("./scripts/parse_traceback.luau");
    let luau_for_traceback = Lua::new();
    globals::set_globals(&luau_for_traceback).unwrap();
    match luau_for_traceback.load(parse_traceback).eval() {
        Ok(LuaValue::Function(parse_traceback)) => {
            parse_traceback.call::<LuaString>(raw_traceback)
                .unwrap()
                .to_string_lossy()
                .to_string()
        },
        Ok(other) => {
            panic!("parse_traceback.luau should return a function??, got: {:#?}", other);
        },
        Err(err) => {
            panic!("parse_traceback.luau broke with err: {err}");
        },
    }
}

#[macro_export]
macro_rules! wrap_err {
    ($msg:expr) => {
        Err(LuaError::external(format!("{}{}{}", colors::RED, $msg, colors::RESET)))
    };
    ($msg:expr, $($arg:tt)*) => {
        Err(LuaError::external(format!("{}{}{}", colors::RED, format!($msg, $($arg)*), colors::RESET)))
    };
}