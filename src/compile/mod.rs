use std::fs;
use std::io::Read;

use crate::prelude::*;
use mluau::prelude::*;
use mluau::Compiler;
use crate::globals;

const BUNDLER_SRC: &str = include_str!("./bundle.luau");

pub fn bundle(project_path: &str) -> LuaResult<String> {
    let luau = Lua::new();
    globals::set_globals(&luau, "bundler")?;
    let bundle = match luau.load(BUNDLER_SRC).eval::<LuaFunction>() {
        Ok(bundle) => bundle,
        Err(err) => {
            panic!("loading seal bundle function broke due to err: {}", err);
        }
    };

    let res = match bundle.call::<LuaValue>(project_path.into_lua(&luau)?) {
        Ok(LuaValue::String(bundled)) => bundled.to_string_lossy(),
        Ok(LuaValue::UserData(ud)) => {
            return wrap_err!("seal bundle - {}", ud.to_string()?)
        },
        Ok(other) => {
            panic!("wtf did seal bundle return? expected string | error, got: {:?}", other);
        }
        Err(err) => {
            panic!("seal bundle errored at runtime: {}", err);
        }
    };

    Ok(res)
}

/// if this seal executable is standalone, returns its compiled bytecode;
/// if it's not standalone, returns None
pub fn extract_bytecode() -> Option<Vec<u8>> {
    const MAGIC: &[u8] = b"ASEALB1N";

    let current_executable = std::env::current_exe().ok()?;
    let executable_bytes = fs::read(&current_executable).ok()?;

    // look for magic header in current exe
    let magic_header_pos = executable_bytes
        .windows(MAGIC.len())
        .rposition(|window| window == MAGIC)?;

    // read bytecode length (exactly 4 bytes from end of magic header)
    let bytecode_len = {
        let len_start = magic_header_pos + MAGIC.len();
        let len_end = len_start + 4;

        if len_end > executable_bytes.len() {
            return None;
        }

        let len_bytes = &executable_bytes[len_start..len_end];
        u32::from_le_bytes(len_bytes.try_into().ok()?) as usize
    };

    // extract bytecode
    let bytecode_start = magic_header_pos + MAGIC.len() + 4;
    let bytecode_end = bytecode_start + bytecode_len;

    if bytecode_end > executable_bytes.len() {
        return None;
    }

    Some(executable_bytes[bytecode_start..bytecode_end].to_vec())
}

pub fn standalone(src: &str) -> LuaResult<Vec<u8>> {
    let comp = Compiler::new();
    let bytecode = match comp.compile(src) {
        Ok(bytecode) => bytecode,
        Err(err) => {
           return wrap_err!("seal compile - unable to compile standalone due to err: {}", err);
        }
    };

    // need to read the current seal executable into memory so we can append magic header and bytecode
    let executable_path = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(err) => {
            return wrap_err!("seal compile - cannot get this seal executable path due to err: {}", err);
        }
    };

    let mut standalone_bytes = Vec::new();
    match fs::File::open(&executable_path)
        .and_then(|mut f| f.read_to_end(&mut standalone_bytes))
    {
        Ok(_) => {},
        Err(err) => {
            return wrap_err!("seal compile - error reading current executable path: {}", err);
        }
    };

    // append magic 8 byte header + length prefix + bytecode
    const MAGIC: &[u8] = b"ASEALB1N";
    let bytecode_len = (bytecode.len() as u32).to_le_bytes();
    standalone_bytes.extend_from_slice(MAGIC);
    standalone_bytes.extend_from_slice(&bytecode_len);
    standalone_bytes.extend_from_slice(&bytecode);

    Ok(standalone_bytes)
}