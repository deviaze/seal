use mluau::prelude::*;
use crate::{prelude::*, std_time::duration::TimeDuration};

use crate::globals::warn;
use crate::std_err::WrappedError;
use rustyline::error::ReadlineError;

use crossterm::event::{Event, KeyEvent, KeyModifiers, MouseEvent};
use crossterm::execute;

use atty::Stream::{Stdout, Stderr, Stdin};

use std::io::{self, Write};
use std::time::Duration;

enum InterruptCode {
    CtrlC,
    CtrlD,
}
pub struct Interrupt {
    code: InterruptCode
}

impl Interrupt {
    pub fn ctrlc() -> Self {
        Self {
            code: InterruptCode::CtrlC
        }
    }
    pub fn ctrld() -> Self {
        Self {
            code: InterruptCode::CtrlD
        }
    }
    pub fn get_userdata(self, luau: &Lua) -> LuaValueResult {
        ok_userdata(self, luau)
    }
}

impl LuaUserData for Interrupt {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_meta_field("__type", "interrupt"); // allow users to typeof check
        fields.add_field_method_get("code", |luau: &Lua, this: &Interrupt| {
            match this.code {
                InterruptCode::CtrlC => "CtrlC".into_lua(luau),
                InterruptCode::CtrlD => "CtrlD".into_lua(luau),
            }
        });
    }
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, | luau: &Lua, this: &Interrupt, _: LuaValue| -> LuaValueResult {
            match this.code {
                InterruptCode::CtrlC => "CtrlC".into_lua(luau),
                InterruptCode::CtrlD => "CtrlD".into_lua(luau),
            }
        });
    }
}

fn input_get(luau: &Lua, raw_prompt: Option<String>) -> LuaValueResult {
    if let Some(prompt) = raw_prompt {
        print!("{}", prompt);
        io::stdout().flush().unwrap();
    }

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim_end().to_string();

    Ok(LuaValue::String(luau.create_string(&input)?))
}

enum WhichTty {
    Stdin,
    Stderr,
    Stdout,
    All,
}
impl WhichTty {
    fn pick(value: LuaValue, function_name: &'static str) -> LuaResult<Self> {
        let s = match value {
            LuaValue::String(ref s) => {
                &*s.as_bytes()
            },
            LuaNil => {
                return Ok(Self::All);
            },
            other => {
                return wrap_err!("{} expected stream to be nil/unspecified or \"Stdout\" | \"Stderr\" | \"Stdin\", got: {:?}", function_name, other);
            }
        };
        match s {
            b"Stdout" => Ok(Self::Stdout),
            b"Stderr" => Ok(Self::Stderr),
            b"Stdin"  => Ok(Self::Stdin),
            _ => {
                wrap_err!("{} expected stream to be nil/unspecified or \"Stdout\" | \"Stderr\" | \"Stdin\", got: {}", function_name, value.as_string().expect("we know it's a string").display())
            }
        }
    }
}

fn is_tty(_luau: &Lua, value: LuaValue) -> LuaResult<bool> {
    let function_name = "io.tty(stream: (\"Stdout\" | \"Stderr\" | \"Stdin\")?)";
    match WhichTty::pick(value, function_name)? {
        WhichTty::All => {
            Ok(atty::is(Stdout) && atty::is(Stderr) && atty::is(Stdin))
        },
        WhichTty::Stdout => Ok(atty::is(Stdout)),
        WhichTty::Stderr => Ok(atty::is(Stderr)),
        WhichTty::Stdin => Ok(atty::is(Stdin)),
    }
}

pub fn input_rawline(_: &Lua, raw_prompt: Option<String>) -> LuaResult<String> {
    if let Some(prompt) = raw_prompt {
        print!("{}", prompt);
        io::stdout().flush()?;
    }

    let mut input= String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim_end().to_string();

    Ok(input)
}

pub fn input_readline(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let function_name = "io.input.readline(prompt: string)";
    let prompt = match multivalue.pop_front() {
        Some(LuaValue::String(s)) => s.to_string_lossy(),
        Some(LuaNil) | None => {
            return wrap_err!("{} expected message to be a string, got nothing or nil", function_name);
        },
        Some(other) => {
            return wrap_err!("{} expected message to be a string, got: {:?}", function_name, other);
        }
    };

    if atty::isnt(atty::Stream::Stdin) || atty::isnt(atty::Stream::Stdout) {
        match input_rawline(luau, if prompt.is_empty() { None } else { Some(prompt) }) {
            Ok(s) => {
                return ok_string(s, luau);
            },
            Err(err) => {
                return wrap_err!("{}: unable to fallback to non-tty due to err: {}", function_name, err);
            }
        }
    }

    let mut rl = match rustyline::DefaultEditor::new() {
        Ok(editor) => editor,
        Err(err) => {
            return wrap_err!("{}: unable to make rustyline DefaultEditor due to ReadlineError: {}", function_name, err);
        }
    };

    let line = match rl.readline(&prompt) {
        Ok(line) => {
            if let Err(err) = rl.add_history_entry(line.as_str()) {
                warn(luau, ok_string(format!("error adding prompt history: {}", err), luau)?)?;
            }
            line
        },
        Err(ReadlineError::Interrupted) => {
            return Interrupt::ctrlc().get_userdata(luau);
        },
        Err(ReadlineError::Eof) => {
            return Interrupt::ctrld().get_userdata(luau);
        },
        Err(ReadlineError::Io(err)) => {
            return WrappedError::from_message(format!("terminal io error: {}", err)).get_userdata(luau);
        }
        Err(err) => {
            return wrap_err!("{}: encountered unexpected ReadlineError: {}", function_name, err);
        }
    };

    ok_string(line, luau)
}

fn input_interrupt(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "input.interrupt(code: \"CtrlC\" | \"CtrlD\")";
    match value {
        LuaValue::String(c) => {
            let bytes = c.as_bytes();
            if bytes == &b"CtrlC"[..] {
                Interrupt::ctrlc().get_userdata(luau)
            } else if bytes == &b"CtrlD"[..] {
                Interrupt::ctrld().get_userdata(luau)
            } else {
                wrap_err!("{} expected code to be \"CtrlC\" or \"CtrlD\", got some other string: {}", function_name, c.display())
            }
        },
        other => {
            wrap_err!("{} expected code to be \"CtrlC\" or \"CtrlD\", got: {:?}", function_name, other)
        }
    }
}

fn input_rawmode(_luau: &Lua, value: LuaValue) -> LuaEmptyResult {
    let function_name = "input.rawmode(enabled: boolean)";
    let enabled = match value {
        LuaValue::Boolean(b) => b,
        other => {
            return wrap_err!("{} expected enabled to be a boolean, got: {:?}", function_name, other);
        }
    };

    if let Err(err) = if enabled {
        crossterm::terminal::enable_raw_mode()
    } else { 
        crossterm::terminal::disable_raw_mode()
    } {
        return wrap_err!("{}: unable to enable/disable raw mode due to err: {}", function_name, err);
    }

    Ok(())
}

fn create_event_table(luau: &Lua, event: Event) -> LuaResult<LuaTable> {
    let t = create_table_with_capacity(luau, 0, 3)?;

    fn table_from_modifiers(luau: &Lua, modifiers: KeyModifiers) -> LuaResult<LuaTable> {
        let t: LuaTable = luau.named_registry_value("InputKeyModifiers")?;
        t.raw_set("shift", modifiers.contains(KeyModifiers::SHIFT))?;
        t.raw_set("ctrl", modifiers.contains(KeyModifiers::CONTROL))?;
        t.raw_set("alt", modifiers.contains(KeyModifiers::ALT))?;
        // t.raw_set("meta", modifiers.contains(KeyModifiers::META))?;
        Ok(t)
    }
    
    match event {
        Event::Key(KeyEvent { code, modifiers, .. }) => {
            t.raw_set("is", "Key")?;
            t.raw_set("key", code.to_string())?;
            t.raw_set("modifiers", table_from_modifiers(luau, modifiers)?)?;
        },
        Event::Mouse(MouseEvent { kind, column, row, modifiers }) => {
            // return ok_string(format!("Mouse: {:?}", kind), luau);
            t.raw_set("is", "Mouse")?;
            t.raw_set("kind", format!("{:?}", kind))?;
            t.raw_set("column", column)?;
            t.raw_set("row", row)?;
            t.raw_set("modifiers", table_from_modifiers(luau, modifiers)?)?;
        },
        Event::FocusLost => {
            t.raw_set("is", "FocusLost")?;
        },
        Event::FocusGained => {
            t.raw_set("is", "FocusGained")?;
        },
        Event::Resize(columns, rows) => {
            t.raw_set("is", "Resize")?;
            t.raw_set("columns", columns)?;
            t.raw_set("rows", rows)?;
        },
        Event::Paste(s) => {
            t.raw_set("is", "Paste")?;
            t.raw_set("contents", s)?;
        },
    }
    t.set_readonly(true);
    Ok(t)
}

fn input_capture_mouse(_luau: &Lua, value: LuaValue) -> LuaEmptyResult {
    let function_name = "input.capture.mouse(enabled: boolean)";
    let should_capture = match value {
        LuaValue::Boolean(b) => b,
        other => {
            return wrap_err!("{} expected enabled to be a boolean, got: {:?}", function_name, other);
        }
    };

    if let Err(err) = if should_capture {
        execute!(
            io::stdout(),
            crossterm::event::EnableMouseCapture,
        )
    } else {
        execute!(
            io::stdout(),
            crossterm::event::DisableMouseCapture,
        )
    } {
        return wrap_err!("{}: cannot {} terminal mouse capture due to err: {}", function_name, if should_capture { "enable" } else { "disable" }, err);
    }
    
    Ok(())
}

fn input_capture_focus(_luau: &Lua, value: LuaValue) -> LuaEmptyResult {
    let function_name = "input.capture.focus(enabled: boolean)";
    let should_capture = match value {
        LuaValue::Boolean(b) => b,
        other => {
            return wrap_err!("{} expected enabled to be a boolean, got: {:?}", function_name, other);
        }
    };

    if let Err(err) = if should_capture {
        execute!(
            io::stdout(),
            crossterm::event::EnableFocusChange,
        )
    } else {
        execute!(
            io::stdout(),
            crossterm::event::DisableFocusChange,
        )
    } {
        return wrap_err!("{}: cannot {} terminal focus capture due to err: {}", function_name, if should_capture { "enable" } else { "disable" }, err);
    }
    
    Ok(())
}

fn input_capture_paste(_luau: &Lua, value: LuaValue) -> LuaEmptyResult {
    let function_name = "input.capture.paste(enabled: boolean)";
    let should_capture = match value {
        LuaValue::Boolean(b) => b,
        other => {
            return wrap_err!("{} expected enabled to be a boolean, got: {:?}", function_name, other);
        }
    };

    if let Err(err) = if should_capture {
        execute!(
            io::stdout(),
            crossterm::event::EnableBracketedPaste,
        )
    } else {
        execute!(
            io::stdout(),
            crossterm::event::DisableBracketedPaste,
        )
    } {
        return wrap_err!("{}: cannot {} terminal bracketed paste capture due to err: {}", function_name, if should_capture { "enable" } else { "disable" }, err);
    }
    
    Ok(())
}

pub fn input_events(luau: &Lua, value: LuaValue) -> LuaResult<LuaFunction> {
    let function_name = "input.events(poll: Duration?)";
    let poll_duration = match value {
        LuaValue::UserData(ud) => {
            if let Ok(duration) = ud.borrow::<TimeDuration>() {
                if duration.inner.is_negative() {
                    return wrap_err!("{}: cannot poll for a negative duration", function_name);
                } else {
                    duration.inner.unsigned_abs()
                }
            } else {
                let type_name = ud.type_name()?.unwrap_or(String::from("userdata"));
                return wrap_err!("{} expected poll to be a Duration, got a different kind of userdata: {}", function_name, type_name);
            }
        },
        LuaNil => {
            Duration::from_millis(50)
        },
        other => {
            return wrap_err!("{} expected poll to be a Duration, got: {:?}", function_name, other);
        }
    };
    
    let empty_event_table = TableBuilder::create(luau)?
        .with_value("is", "Empty")?
        .build_readonly()?;

    let modifiers_table = create_table_with_capacity(luau, 0, 3)?;
    luau.set_named_registry_value("InputKeyModifiers", modifiers_table)?;

    let empty_event_registry_key = luau.create_registry_value(empty_event_table)?;

    let f = luau.create_function(move | luau: &Lua, _: LuaValue | -> LuaValueResult {
        if let Ok(b) = crossterm::event::poll(poll_duration) && b {
            let event =  match crossterm::event::read() {
                Ok(event) => event,
                Err(err) => {
                    return wrap_err!("event not eventing due to err: {}", err);
                }
            };
            ok_table(create_event_table(luau, event))
        } else {
            let empty_event_table: LuaTable = luau.registry_value(&empty_event_registry_key)?;
            ok_table(Ok(empty_event_table))
        }
    })?;

    Ok(f)
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("get", input_get)?
        .with_function("tty", is_tty)?
        .with_function("rawmode", input_rawmode)?
        .with_function("readline", input_readline)?
        .with_function("rawline", input_rawline)?
        .with_function("interrupt", input_interrupt)?
        .with_function("events", input_events)?
        .with_value("capture", TableBuilder::create(luau)?
            .with_function("mouse", input_capture_mouse)?
            .with_function("focus", input_capture_focus)?
            .with_function("paste", input_capture_paste)?
            .build_readonly()?
        )?
        .build_readonly()
}
