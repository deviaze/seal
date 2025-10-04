// pub use crate::{LuaValueResult, LuaEmptyResult, LuaMultiResult, colors, wrap_err};
use mluau::prelude::*;

pub const MAX_TABLE_SIZE: usize = 134_217_728;

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

pub fn ok_function_mut<F, I, Fn>(f: Fn, luau: &Lua) -> LuaValueResult
where
    F: FromLuaMulti + 'static,
    I: IntoLuaMulti + 'static,
    Fn: FnMut(&Lua, F) -> LuaResult<I> + 'static,
{
    Ok(LuaValue::Function(luau.create_function_mut(f)?))
}

pub fn ok_string<S: AsRef<[u8]>>(s: S, luau: &Lua) -> LuaValueResult {
    Ok(LuaValue::String(luau.create_string(s)?))
}

pub fn ok_buffy<B: AsRef<[u8]>>(b: B, luau: &Lua) -> LuaValueResult {
    Ok(LuaValue::Buffer(luau.create_buffer(b)?))
}

pub fn ok_userdata<S: LuaUserData + Send + 'static>(u: S, luau: &Lua) -> LuaValueResult {
    Ok(LuaValue::UserData(luau.create_userdata(u)?))
}

pub fn pop_self(multivalue: &mut LuaMultiValue, function_name: &'static str) -> LuaEmptyResult {
    match multivalue.pop_front() {
        Some(LuaValue::Table(_s)) => Ok(()),
        Some(other) => {
            wrap_err!("{} expected to be called with self, got: {:?}; did you forget to use methodcall syntax (:)?", function_name, other)
        },
        None => {
            wrap_err!("{} incorrectly called with zero arguments, expected self", function_name)
        }
    }
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
        let caller_function_name = match info.raw_get("function_name")? {
            LuaValue::String(s) => s.to_string_lossy(),
            LuaNil => String::from("<FUNCTION NAME NOT FOUND>"),
            other => {
                return wrap_err!("{}: expected function_name to be a string, got: {:?}", function_name, other);
            }
        };

        Ok(Self { source, line, function_name: caller_function_name })
    }
}


/// safely convert i64 to usize while handling common problems like negatives and out of ranges
pub fn int_to_usize(i: i64, function_name: &str, parameter_name: &'static str) -> LuaResult<usize> {
    if i.is_negative() {
        return wrap_err!("{}: {} represents a byte offset or countable number and cannot be negative (got {})", function_name, parameter_name, i);
    }
    match usize::try_from(i) {
        Ok(u) => Ok(u),
        Err(err) => {
            wrap_err!("{}: {} can't safely be converted from i64 to usize because {}", function_name, parameter_name, err)
        }
    }
}

pub fn int_to_u64(i: i64, function_name: &'static str, parameter_name: &'static str) -> LuaResult<u64> {
    if i.is_negative() {
        return wrap_err!("{}: {} must be positive (got: {})", function_name, parameter_name, i);
    }
    match u64::try_from(i) {
        Ok(u) => Ok(u),
        Err(err) => {
            wrap_err!("{}: {} can't safely be converted from i64 to u64 because {}", function_name, parameter_name, err)
        }
    }
}

/// safely convert float param to usize, giving a good error reason if it didn't successfully convert
pub fn float_to_usize(f: f64, function_name: &'static str, parameter_name: &'static str) -> LuaResult<usize> {
    let truncated = f.trunc();
    if truncated.is_nan() || truncated.is_infinite() {
        wrap_err!("{}: {} cannot be NaN nor infinite", function_name, parameter_name)
    } else if truncated.is_sign_negative() {
        wrap_err!("{}: {} represents a byte offset and cannot be negative (got: {})", function_name, parameter_name, truncated)
    } else if truncated > usize::MAX as f64 {
        wrap_err!("{}: expected {} to be convertible into usize, however provided float is too big to fit (got: {})", function_name, parameter_name, f)
    } else if truncated == f {
        // SAFETY: we just checked nan/infinite/size/negative right above
        let i = unsafe { truncated.to_int_unchecked() };
        int_to_usize(i, function_name, parameter_name)
    } else {
        wrap_err!("{} expected {} to be an integer number, unexpectedly got a float: {}", function_name, parameter_name, f)
    }
}

/// safely convert float param to u64, giving a good error reason if conversion wasn't successful
pub fn float_to_u64(f: f64, function_name: &'static str, parameter_name: &'static str) -> LuaResult<u64> {
    let truncated = f.trunc();
    if truncated.is_nan() || truncated.is_infinite() {
        wrap_err!("{}: {} cannot be NaN nor infinite", function_name, parameter_name)
    } else if truncated.is_sign_negative() {
        wrap_err!("{}: {} cannot be negative (got: {})", function_name, parameter_name, f)
    } else if truncated > u64::MAX as f64 {
        wrap_err!("{} expected {} to be convertible to u64, but provided float is too big to fit (got: {})", function_name, parameter_name, f)
    } else if truncated == f {
        // SAFETY: just checked nan/infinite/size/negative right above
        let u: u64 = unsafe { truncated.to_int_unchecked() };
        Ok(u)
    } else {
        wrap_err!("{} expected {} to be an integer, unexpectedly got a float: {}", function_name, parameter_name, f)
    }
}

/// Creates table with capacity, clamping upper capacity to `MAX_TABLE_SIZE` for safety
pub fn create_table_with_capacity(luau: &Lua, n_array: usize, n_records: usize) -> LuaResult<LuaTable> {
    let n_array = std::cmp::min(n_array, MAX_TABLE_SIZE);
    let n_records = std::cmp::min(n_records, MAX_TABLE_SIZE);
    // SAFETY: luau.create_table_with_capacity will abort if `capacity` exceeds MAX_TABLE_SIZE (throwing Rust cannot catch foreign exceptions)
    // We clamp `good_prealloc_guess` to MAX_TABLE_SIZE to guarantee safety.
    // This API should be marked unsafe... but isn't.. so we explicitly treat it as unsafe here.
    #[allow(unused_unsafe)]
    unsafe { luau.create_table_with_capacity(n_array, n_records) }
}