use mlua::prelude::*;
use crate::prelude::*;

pub fn testing_try(luau: &Lua, f: LuaValue) -> LuaValueResult {
    match f {
        LuaValue::Function(f) => {
            let pcall: LuaFunction = luau.globals().get("pcall")?;
            let mut result  = pcall.call::<LuaMultiValue>(f)?;
            let success = result.pop_front().unwrap();
            let result = result.pop_front().unwrap_or(LuaNil);

            let result_table = luau.create_table()?;

            if let LuaValue::Boolean(success) = success {
                if success {
                    result_table.set("ok", true)?;
                    result_table.set("result", result)?;
                    result_table.set("match", luau.create_function(
                        |_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                            let self_table = match multivalue.pop_front().unwrap() {
                                LuaValue::Table(self_table) => self_table,
                                other => {
                                    return wrap_err!("TryResult:match() expected self to be a TryResult, got: {:?}", other);
                                }
                            };
                            let handler = multivalue.pop_back().unwrap();
                            match handler {
                                LuaValue::Table(handler) => {
                                    let ok_result: LuaValue = self_table.get("result")?;
                                    let ok_handler: LuaValue = handler.get("ok")?;
                                    if let LuaValue::Function(ok_handler) = ok_handler {
                                        Ok(ok_handler.call::<LuaValue>(ok_result)?)
                                    } else {
                                        Ok(ok_handler)
                                    }
                                },
                                other => {
                                    wrap_err!("TryResult:match() expected handler to be a table with field 'ok', got {:?}", other)
                                }
                            }
                        }
                    )?)?;
                    result_table.set("unwrap", luau.create_function(
                        |_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                            match multivalue.pop_front() {
                                Some(LuaValue::Table(self_table)) => {
                                    let result = self_table.get("result")?;
                                    Ok(result)
                                },
                                None => {
                                    wrap_err!("TryResult:unwrap() was called with zero arguments (not even self)")
                                },
                                other => {
                                    wrap_err!("TryResult:unwrap() wtf is self?: {:?}", other)
                                }
                            }
                        }
                    )?)?;
                    result_table.raw_set("expect_err", luau.create_function(
                        |_luau: &Lua, value: LuaValue| -> LuaValueResult {
                            wrap_err!("Error expected, but the function call didn't error. Instead, we got: {:#?}", value)
                        }
                    )?)?;
                } else {
                    result_table.set("ok", false)?;
                    result_table.set("err", result.clone())?;
                    result_table.set("match", luau.create_function(
                        |_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                            let self_table = match multivalue.pop_front().unwrap() {
                                LuaValue::Table(self_table) => self_table,
                                other => {
                                    return wrap_err!("TryResult:match() expected self to be self, got: {:?}", other);
                                }
                            };
                            let handler = multivalue.pop_back().unwrap();
                            match handler {
                                LuaValue::Table(handler) => {
                                    let err_result: LuaValue = self_table.get("err")?;
                                    let err_handler: LuaValue = handler.get("err")?;
                                    if let LuaValue::Function(ok_handler) = err_handler {
                                        Ok(ok_handler.call::<LuaValue>(err_result)?)
                                    } else {
                                        Ok(err_handler)
                                    }
                                },
                                other => {
                                    wrap_err!("TryResult:match() expected handler to be a table with field 'err', got {:?}", other)
                                }
                            }
                        }
                    )?)?;
                    result_table.set("unwrap", luau.create_function(
                        |_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                            match multivalue.pop_front() {
                                Some(LuaValue::Table(self_table)) => {
                                    let err_result: LuaError = self_table.get("err")?;
                                    match multivalue.pop_front() {
                                        Some(default_value) => {
                                            Ok(default_value)
                                        },
                                        None => {
                                            wrap_err!("Attempted to :unwrap() a TryResult without a default value! Error: \n  {}", err_result.to_string())
                                        }
                                    }
                                },
                                None => {
                                    wrap_err!("TryResult:unwrap() was called with zero arguments (not even self)")
                                },
                                other => {
                                    wrap_err!("TryResult:unwrap() wtf is self?: {:?}", other)
                                }
                            }
                        }
                    )?)?;
                    result_table.set("expect_err", luau.create_function(
                        move |luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                            let _s = multivalue.pop_front();
                            let handler = match multivalue.pop_front() {
                                Some(handler) => handler,
                                None => LuaValue::Boolean(true),
                            };

                            match handler {
                                LuaValue::Function(handler) => {
                                    // call the handler function and return whatever it returns
                                    Ok(handler.call::<LuaValue>(result.clone())?)
                                },
                                LuaValue::String(error_matcher) => {
                                    // match error_matcher against the error we got using luau's string.match
                                    let string_lib: LuaTable = luau.globals().raw_get("string")?;
                                    let string_match: LuaFunction = string_lib.raw_get("match")?;

                                    // let global_tostring: LuaFunction = luau.globals().raw_get("tostring")?;
                                    // let stringified_err: LuaValue = global_tostring.call(result.clone())?;
                                    let stringified_err = crate::std_io::output::strip_newlines_and_colors(&result.to_string()?);

                                    let match_args_vec: Vec<LuaValue> = vec!(
                                        luau.create_string(&stringified_err)?.into_lua(luau)?, 
                                        error_matcher.to_owned().into_lua(luau)?
                                    );
                            
                                    let match_args = LuaMultiValue::from_vec(match_args_vec);

                                    let does_match: bool = match string_match.call::<LuaValue>(match_args)? {
                                        LuaValue::String(_s) => true,
                                        _other => false,
                                    };

                                    if does_match {
                                        Ok(LuaValue::Boolean(true))
                                    } else {
                                        wrap_err!("Error did not match the expected error! Expected err to string.match {:#?}, got err:\n  {}", error_matcher, stringified_err)
                                    }
                                },
                                other => {
                                    wrap_err!("Expected error handler to be a function or a string (to string.match an error against), got: {:#?}", other)
                                }
                            }
                        }
                    )?)?;
                }
            } else {
                unreachable!("wtf else is success other than boolean??? {:?}", success);
            };

            Ok(LuaValue::Table(result_table))
        },
        other => {
            wrap_err!("try expected a function, got {:?}", other)
        }
    }
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
	TableBuilder::create(luau)?
		.with_function("try", testing_try)?
		.build()
}