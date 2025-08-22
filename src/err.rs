use std::panic;
use mluau::prelude::*;
use crate::globals;
use crate::prelude::*;

#[cfg(unix)]
use libc::{signal, SIGABRT, exit};

#[cfg(unix)]
unsafe extern "C" fn handle_sigabrt(_: i32) {
    unsafe {
        exit(1);
    }
}

/// Intercepts SIGABRT on Unix-like systems to prevent core dumps and keep `seal` as nonblocking as possible.
pub fn setup_sigabrt_handler() {
    // Intercepts SIGABRT to prevent core dumps when `seal` is used as a child process.
    // This occurs because `ChildProcessStream` readers are fully nonblocking;
    // if the parent process exits while either `ChildProcessStream` reader is `reader.read()`ing to the inner buffer,
    // the child process may exit with a SIGABRT, causing a core dump--even though `seal` itself is functioning correctly.
    // We intercept these core dumps to prevent leaking sensitive memory.
    //
    // SAFETY:
    // - This signal is registered as early as possible in `main`.
    // - It is the only signal handler registered by `seal`; registering any other signal handlers may cause UB.
    // - Only async-signal-safe functions (such as `exit()`) are called in the signal handler.
    // - Behavior has been tested on x86_64 Arch Linux.
    #[cfg(unix)]
    unsafe {
        signal(SIGABRT, handle_sigabrt as usize);
    }
}

pub fn parse_traceback(raw_traceback: String) -> String {
    let parse_traceback = include_str!("./scripts/parse_traceback.luau");
    let luau_for_traceback = Lua::new();
    globals::set_globals(&luau_for_traceback, String::default()).unwrap();
    match luau_for_traceback.load(parse_traceback).eval() {
        Ok(LuaValue::Function(parse_traceback)) => {
            parse_traceback.call::<LuaString>(raw_traceback)
                .expect("calling parse_traceback.luau should not fail unexpectedly.")
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

pub fn setup_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let payload = info.payload().downcast_ref::<&str>().map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "Unknown error running the custom panic hook, please report this to the manager (deviaze)".to_string());
        
        // eprintln!("{}[ERR]{}{} {}{}", colors::BOLD_RED, colors::RESET, colors::RED, payload, colors::RESET);
        eprintln!("{}[PANIC! IN THE SEAL INTERNALS]: {}{}{}{}", colors::BOLD_RED, colors::RESET, colors::YELLOW, payload, colors::RESET);
        if let Some(location) = info.location() {
            eprintln!("panic occurred at: [\"{}\"]:{}:{}", location.file(), location.line(), location.column());
        }
        eprintln!(
            "{}\nseal panicked. seal is not supposed to panic, so you've found a bug.\nPlease report it here: https://github.com/deviaze/seal{}",
            colors::RED, colors::RESET
        );
    }));
}

pub fn display_error_and_exit(err: LuaError) -> ! {
    let err = parse_traceback(err.to_string());
    eprintln!("{}[ERR]{} {}", colors::BOLD_RED, colors::RESET, err);
    std::process::exit(1);
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