// pub use crate::{LuaValueResult, LuaEmptyResult, LuaMultiResult, colors, wrap_err};
use mluau::prelude::*;

pub use crate::{std_io::colors as colors, wrap_err, table_helpers::TableBuilder};

pub type LuaValueResult = LuaResult<LuaValue>;
pub type LuaEmptyResult = LuaResult<()>;
pub type LuaMultiResult = LuaResult<LuaMultiValue>;

// wraps returns of stdlib::create functions with Ok(LuaValue::Table(t))
pub fn ok_table(t: LuaResult<LuaTable>) -> LuaValueResult {
    Ok(LuaValue::Table(t?))
}

pub fn ok_function(f: fn(&Lua, LuaValue) -> LuaValueResult, luau: &Lua) -> LuaValueResult {
    Ok(LuaValue::Function(luau.create_function(f)?))
}

pub fn ok_string<S: AsRef<[u8]>>(s: S, luau: &Lua) -> LuaValueResult {
    Ok(LuaValue::String(luau.create_string(s)?))
}

pub fn ok_buffy<B: AsRef<[u8]>>(b: B, luau: &Lua) -> LuaValueResult {
    Ok(LuaValue::Buffer(luau.create_buffer(b)?))
}

pub struct DebugInfo {
    pub source: String,
    pub line: String,
    pub function_name: String,
}
impl DebugInfo {
    /// returns location info from the luau function that called the current (presumably rust) function
    pub fn from_caller(luau: &Lua, function_name: &'static str) -> LuaResult<Self> {
        const SLN_SRC: &str = r#"
            local source, line, function_name = debug.info(3, "sln")
            return {
                source = source,
                line = line,
                function_name = if function_name == "" then "top level" else function_name,
            }
        "#;
        let LuaValue::Table(info) = luau.load(SLN_SRC).set_name("gettin da debug info").eval()? else {
            return wrap_err!("{}: can't get debug info", function_name);
        };
        let source = match info.raw_get("source")? {
            LuaValue::String(s) => s.to_string_lossy(),
            LuaNil => String::from("<SOURCE NOT FOUND>"),
            other => {
                return wrap_err!("{}: expected source to be a string, got: {:?}", function_name, other);
            }
        };
        let line = match info.raw_get("line")? {
            LuaValue::Integer(n) => n.to_string(),
            LuaNil => String::from("<LINE NOT FOUND>"),
            other => {
                return wrap_err!("{}: expected line, got: {:?}", function_name, other);
            }
        };
        let function_name = match info.raw_get("function_name")? {
            LuaValue::String(s) => s.to_string_lossy(),
            LuaNil => String::from("<FUNCTION NAME NOT FOUND>"),
            other => {
                return wrap_err!("{}: expected function_name to be a string, got: {:?}", function_name, other);
            }
        };

        Ok(Self { source, line, function_name })
    }
}
