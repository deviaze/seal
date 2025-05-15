use aho_corasick::AhoCorasick;
use unicode_segmentation::UnicodeSegmentation;

use std::io::Cursor;
use unicode_reader::Graphemes;

use mlua::prelude::*;
use crate::prelude::*;

/// str.split is an improvement on luau's string.split in that you can split on multiple different choices of characters/strings
/// (not just a single string) and that the splitting is fully unicode grapheme aware
/// by default, str.split splits the string by unicode characters (graphemes)
fn str_split(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "str.split(s: string, chars/graphemes: ...string)";
    let s_bytes = match multivalue.pop_front() {
        Some(LuaValue::String(s)) => {
            s.as_bytes().to_owned()
        },
        Some(other) => {
            return wrap_err!("{} expected s to be a string, got: {:?}", function_name, other);
        }
        None => {
            return wrap_err!("{} expected a string s, but was incorrectly called with zero arguments", function_name);
        }
    };

    // we want to assume most graphemes are normal ascii but some could be multibyte 
    // so 3/4th of the byte len is a good compromise
    let good_prealloc_guess = s_bytes.len() * 3 / 4;
    let result = if multivalue.is_empty() {
        let result = luau.create_table_with_capacity(good_prealloc_guess, 0)?;
        // check if whole string is valid utf-8
        if let Ok(s_str) = std::str::from_utf8(&s_bytes) {
            for grapheme in s_str.graphemes(true) {
                result.raw_push(luau.create_string(grapheme)?)?;
            }
        } else {
            // uh oh we have a mix of graphemes and invalid utf8
            for chunk in s_bytes.utf8_chunks() {
                for grapheme in chunk.valid().graphemes(true) {
                    result.raw_push(luau.create_string(grapheme)?)?;
                }
                if !chunk.invalid().is_empty() {
                    result.raw_push(luau.create_string(chunk.invalid())?)?;
                }
            }
        }
        result
    } else {
        let mut separators = Vec::new();
        for (index, value) in multivalue.iter().enumerate() {
            match value {
                LuaValue::String(sep) => {
                    separators.push(sep.as_bytes().to_owned());
                },
                other => {
                    return wrap_err!("{}: separator at index {} is not a string; got: {:?}", function_name, index, other);
                }
            }
        }
        // if we're splitting by separators, it's probably a comma, \n, or something else
        // we likely won't have as many results as we would when splitting by graphemes, so we preallocate less
        let result = luau.create_table_with_capacity(s_bytes.len() / 2, 0)?;
        let ac =  match AhoCorasick::new(separators) {
            Ok(ac) => ac,
            Err(err) => {
                return wrap_err!("{}: can't initialize AhoCorasick matcher (with separators) due to err: {}", function_name, err);
            }
        };

        let ac_iterator =  match ac.try_find_iter(&s_bytes) {
            Ok(iterator) => iterator,
            Err(err) => {
                return wrap_err!("{}: can't generate AhoCorasick iterator due to err: {}", function_name, err);
            }
        };

        let mut prev_index = 0;
        for mat in ac_iterator {
            let mat_start = mat.start();
            let mat_end = mat.end();
            if mat_start >= prev_index {
                let slice = &s_bytes[prev_index..mat_start];
                if !slice.is_empty() {
                    result.raw_push(luau.create_string(slice)?)?;
                }
            }
            prev_index = mat_end;
        }

        let s_len = s_bytes.len();
        if prev_index < s_len {
            // final element excluded (between last match and end of string) so let's push it manually
            result.raw_push(luau.create_string(&s_bytes[prev_index..s_len])?)?;
        }

        result
    };
    Ok(LuaValue::Table(result))
}

type GraphemePair = Option<(usize, String)>;
/// returns an iterator function you can call multiple times to get the next grapheme of String `s` 
/// without loading the whole String into memory
fn graphemes(s: String, function_name: &'static str) -> Box<dyn FnMut() -> LuaResult<GraphemePair>> {
    let mut current_byte: usize = 0;
    let s_cursor = Cursor::new(s);
    let mut graphemes = Graphemes::from(s_cursor);

    let stateful_iter_fn = move || -> LuaResult<GraphemePair> {
        let Some(result) = graphemes.next() else {
            return Ok(None);
        };
        match result {
            Ok(grapheme) => {
                let r = (current_byte, grapheme);
                current_byte += 1;
                Ok(Some(r))
            },
            Err(err) => {
                wrap_err!(
                    "{}: bytes around and after {} could not be converted to utf-8 graphemes: {}", 
                    function_name, current_byte, err
                )
            }
        }
    };

    Box::new(stateful_iter_fn)
}

fn str_graphemes(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "str.graphemes(s: string)";
    let s = match value {
        LuaValue::String(s) => match s.to_str() {
            Ok(s) => s.to_owned(),
            Err(_) => {
                return wrap_err!("{} expected s to be a valid utf-8 encoded string", function_name);
            }
        },
        other => {
            return wrap_err!("{} expected s to be a string, got: {:?}", function_name, other);
        }
    };

    let mut next_grapheme = graphemes(s, function_name);

    let iter_fn = 
        luau.create_function_mut(move | luau: &Lua, _value: LuaValue| -> LuaMultiResult 
    {
        let Some((current_byte, grapheme)) = next_grapheme()? else {
            return Ok(LuaMultiValue::from_vec(vec![LuaNil]));
        };

        let current_byte_for_luau = match i64::try_from(current_byte) {
            Ok(i) => i + 1, // we want to align start with string.sub start in luau
            Err(_) => {
                return wrap_err!("{}: usize too big to fit in i64 :(", function_name)
            }
        };

        Ok(LuaMultiValue::from_vec(vec![
            LuaValue::Integer(current_byte_for_luau),
            LuaValue::String(luau.create_string(&grapheme)?)
        ]))
    })?;

    Ok(LuaValue::Function(iter_fn))
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("split", str_split)?
        .with_function("graphemes", str_graphemes)?
        .build_readonly()
}