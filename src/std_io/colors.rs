#![allow(dead_code)]

use mluau::prelude::*;
use crate::table_helpers::TableBuilder;

pub const RESET: &str = "\x1b[0m";
pub const BLACK: &str = "\x1b[30m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const WHITE: &str = "\x1b[37m";

pub const BOLD_BLACK: &str = "\x1b[1;30m";
pub const BOLD_RED: &str = "\x1b[1;31m";
pub const BOLD_GREEN: &str = "\x1b[1;32m";
pub const BOLD_YELLOW: &str = "\x1b[1;33m";
pub const BOLD_BLUE: &str = "\x1b[1;34m";
pub const BOLD_MAGENTA: &str = "\x1b[1;35m";
pub const BOLD_CYAN: &str = "\x1b[1;36m";
pub const BOLD_WHITE: &str = "\x1b[1;37m";

pub const BRIGHT_BLACK: &str = "\x1b[90m";
pub const BRIGHT_RED: &str = "\x1b[91m";
pub const BRIGHT_GREEN: &str = "\x1b[92m";
pub const BRIGHT_YELLOW: &str = "\x1b[93m";
pub const BRIGHT_BLUE: &str = "\x1b[94m";
pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
pub const BRIGHT_CYAN: &str = "\x1b[96m";
pub const BRIGHT_WHITE: &str = "\x1b[97m";

pub const BLACK_BG: &str = "\x1b[40m";
pub const RED_BG: &str = "\x1b[41m";
pub const GREEN_BG: &str = "\x1b[42m";
pub const YELLOW_BG: &str = "\x1b[43m";
pub const BLUE_BG: &str = "\x1b[44m";
pub const MAGENTA_BG: &str = "\x1b[45m";
pub const CYAN_BG: &str = "\x1b[46m";
pub const WHITE_BG: &str = "\x1b[47m";

pub const BRIGHT_BLACK_BG: &str = "\x1b[100m";
pub const BRIGHT_RED_BG: &str = "\x1b[101m";
pub const BRIGHT_GREEN_BG: &str = "\x1b[102m";
pub const BRIGHT_YELLOW_BG: &str = "\x1b[103m";
pub const BRIGHT_BLUE_BG: &str = "\x1b[104m";
pub const BRIGHT_MAGENTA_BG: &str = "\x1b[105m";
pub const BRIGHT_CYAN_BG: &str = "\x1b[106m";
pub const BRIGHT_WHITE_BG: &str = "\x1b[107m";

pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const UNDERLINE: &str = "\x1b[4m";

type LuaValueResult = LuaResult<LuaValue>;

fn colorize(luau: &Lua, text: String, color_code: &str) -> LuaValueResult {
    let colored_text = color_code.to_string() + &text + RESET;
    Ok(LuaValue::String(luau.create_string(&colored_text)?))
}

fn colorize_black(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, BLACK)
}

fn colorize_red(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, RED)
}

fn colorize_green(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, GREEN)
}

fn colorize_yellow(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, YELLOW)
}

fn colorize_blue(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, BLUE)
}

fn colorize_magenta(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, MAGENTA)
}

fn colorize_cyan(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, CYAN)
}

fn colorize_white(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, WHITE)
}

fn colorize_bold_black(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, BOLD_BLACK)
}

fn colorize_bold_red(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, BOLD_RED)
}

fn colorize_bold_green(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, BOLD_GREEN)
}

fn colorize_bold_yellow(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, BOLD_YELLOW)
}

fn colorize_bold_blue(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, BOLD_BLUE)
}

fn colorize_bold_magenta(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, BOLD_MAGENTA)
}

fn colorize_bold_cyan(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, BOLD_CYAN)
}

fn colorize_bold_white(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, BOLD_WHITE)
}

fn style_bold(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, BOLD)
}

fn style_dim(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, DIM)
}

fn style_underline(luau: &Lua, text: String) -> LuaValueResult {
    colorize(luau, text, UNDERLINE)
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    let bold_colors = TableBuilder::create(luau)?
        .with_function("black", colorize_bold_black)?
        .with_function("red", colorize_bold_red)?
        .with_function("green", colorize_bold_green)?
        .with_function("yellow", colorize_bold_yellow)?
        .with_function("blue", colorize_bold_blue)?
        .with_function("magenta", colorize_bold_magenta)?
        .with_function("cyan", colorize_bold_cyan)?
        .with_function("white", colorize_bold_white)?
        .build_readonly()?;

    let styles = TableBuilder::create(luau)?
        .with_function("bold", style_bold)?
        .with_function("dim", style_dim)?
        .with_function("underline", style_underline)?
        .build_readonly()?;

    let codes = TableBuilder::create(luau)?
        .with_value("RESET", RESET)?
        .with_value("BLACK", BLACK)?
        .with_value("RED", RED)?
        .with_value("GREEN", GREEN)?
        .with_value("YELLOW", YELLOW)?
        .with_value("BLUE", BLUE)?
        .with_value("MAGENTA", MAGENTA)?
        .with_value("CYAN", CYAN)?
        .with_value("WHITE", WHITE)?
        .with_value("BOLD_BLACK", BOLD_BLACK)?
        .with_value("BOLD_RED", BOLD_RED)?
        .with_value("BOLD_GREEN", BOLD_GREEN)?
        .with_value("BOLD_YELLOW", BOLD_YELLOW)?
        .with_value("BOLD_BLUE", BOLD_BLUE)?
        .with_value("BOLD_MAGENTA", BOLD_MAGENTA)?
        .with_value("BOLD_CYAN", BOLD_CYAN)?
        .with_value("BOLD_WHITE", BOLD_WHITE)?
        .with_value("BRIGHT_BLACK", BRIGHT_BLACK)?
        .with_value("BRIGHT_RED", BRIGHT_RED)?
        .with_value("BRIGHT_GREEN", BRIGHT_GREEN)?
        .with_value("BRIGHT_YELLOW", BRIGHT_YELLOW)?
        .with_value("BRIGHT_BLUE", BRIGHT_BLUE)?
        .with_value("BRIGHT_MAGENTA", BRIGHT_MAGENTA)?
        .with_value("BRIGHT_CYAN", BRIGHT_CYAN)?
        .with_value("BRIGHT_WHITE", BRIGHT_WHITE)?
        .with_value("BLACK_BG", BLACK_BG)?
        .with_value("RED_BG", RED_BG)?
        .with_value("GREEN_BG", GREEN_BG)?
        .with_value("YELLOW_BG", YELLOW_BG)?
        .with_value("BLUE_BG", BLUE_BG)?
        .with_value("MAGENTA_BG", MAGENTA_BG)?
        .with_value("CYAN_BG", CYAN_BG)?
        .with_value("WHITE_BG", WHITE_BG)?
        .with_value("BRIGHT_BLACK_BG", BRIGHT_BLACK_BG)?
        .with_value("BRIGHT_RED_BG", BRIGHT_RED_BG)?
        .with_value("BRIGHT_GREEN_BG", BRIGHT_GREEN_BG)?
        .with_value("BRIGHT_YELLOW_BG", BRIGHT_YELLOW_BG)?
        .with_value("BRIGHT_BLUE_BG", BRIGHT_BLUE_BG)?
        .with_value("BRIGHT_MAGENTA_BG", BRIGHT_MAGENTA_BG)?
        .with_value("BRIGHT_CYAN_BG", BRIGHT_CYAN_BG)?
        .with_value("BRIGHT_WHITE_BG", BRIGHT_WHITE_BG)?
        .with_value("BOLD", BOLD)?
        .with_value("DIM", DIM)?
        .with_value("UNDERLINE", UNDERLINE)?
        .build_readonly()?;

    TableBuilder::create(luau)?
        .with_function("black", colorize_black)?
        .with_function("red", colorize_red)?
        .with_function("green", colorize_green)?
        .with_function("yellow", colorize_yellow)?
        .with_function("blue", colorize_blue)?
        .with_function("magenta", colorize_magenta)?
        .with_function("cyan", colorize_cyan)?
        .with_function("white", colorize_white)?
        .with_value("bold", bold_colors)?
        .with_value("style", styles)?
        .with_value("codes", codes)?
        .build_readonly()
}
