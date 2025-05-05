--[=[
The `@std/io/colors` lib, because if your terminal output isn't colorized, is it even output?

Usage:

```luau
local colors = require("@std/io/colors")
```
]=]

export type colors = {
    black: (text: string) -> string,
    red: (text: string) -> string,
    green: (text: string) -> string,
    yellow: (text: string) -> string,
    blue: (text: string) -> string,
    magenta: (text: string) -> string,
    cyan: (text: string) -> string,
    white: (text: string) -> string,

    bold: {
        black: (text: string) -> string,
        red: (text: string) -> string,
        green: (text: string) -> string,
        yellow: (text: string) -> string,
        blue: (text: string) -> string,
        magenta: (text: string) -> string,
        cyan: (text: string) -> string,
        white: (text: string) -> string,
    },

    style: {
        dim: (text: string) -> string,
        bold: (text: string) -> string,
    },

    codes: {
        RESET: "\x1b[0m",
        BLACK: "\x1b[30m",
        RED: "\x1b[31m",
        GREEN: "\x1b[32m",
        YELLOW: "\x1b[33m",
        BLUE: "\x1b[34m",
        MAGENTA: "\x1b[35m",
        CYAN: "\x1b[36m",
        WHITE: "\x1b[37m",
        BOLD_BLACK: "\x1b[1;30m",
        BOLD_RED: "\x1b[1;31m",
        BOLD_GREEN: "\x1b[1;32m",
        BOLD_YELLOW: "\x1b[1;33m",
        BOLD_BLUE: "\x1b[1;34m",
        BOLD_MAGENTA: "\x1b[1;35m",
        BOLD_CYAN: "\x1b[1;36m",
        BOLD_WHITE: "\x1b[1;37m",
        BRIGHT_BLACK: "\x1b[90m",
        BRIGHT_RED: "\x1b[91m",
        BRIGHT_GREEN: "\x1b[92m",
        BRIGHT_YELLOW: "\x1b[93m",
        BRIGHT_BLUE: "\x1b[94m",
        BRIGHT_MAGENTA: "\x1b[95m",
        BRIGHT_CYAN: "\x1b[96m",
        BRIGHT_WHITE: "\x1b[97m",
        BLACK_BG: "\x1b[40m",
        RED_BG: "\x1b[41m",
        GREEN_BG: "\x1b[42m",
        YELLOW_BG: "\x1b[43m",
        BLUE_BG: "\x1b[44m",
        MAGENTA_BG: "\x1b[45m",
        CYAN_BG: "\x1b[46m",
        WHITE_BG: "\x1b[47m",
        BRIGHT_BLACK_BG: "\x1b[100m",
        BRIGHT_RED_BG: "\x1b[101m",
        BRIGHT_GREEN_BG: "\x1b[102m",
        BRIGHT_YELLOW_BG: "\x1b[103m",
        BRIGHT_BLUE_BG: "\x1b[104m",
        BRIGHT_MAGENTA_BG: "\x1b[105m",
        BRIGHT_CYAN_BG: "\x1b[106m",
        BRIGHT_WHITE_BG: "\x1b[107m",
        BOLD: "\x1b[1m",
        DIM: "\x1b[2m",
    }
}

-- yeah so apparently you can't put moonwave docs in typedefs so please
-- don't pray for my sanity wrt. the below code because even tho copilot couldn't handle it 
-- i used a luau script (with this runtime ofc)

local colors = {}
--[=[
Turns the provided text magenta

Usage:

```luau
local colors = require("@std/io/colors")
print(colors.magenta("this text is hereby colored magenta"))
```
]=]
function colors.magenta(text: string): string
    return nil :: any
end
--[=[
Turns the provided text blue

Usage:

```luau
local colors = require("@std/io/colors")
print(colors.blue("this text is hereby colored blue"))
```
]=]
function colors.blue(text: string): string
    return nil :: any
end
--[=[
Turns the provided text cyan

Usage:

```luau
local colors = require("@std/io/colors")
print(colors.cyan("this text is hereby colored cyan"))
```
]=]
function colors.cyan(text: string): string
    return nil :: any
end
--[=[
Turns the provided text black

Usage:

```luau
local colors = require("@std/io/colors")
print(colors.black("this text is hereby colored black"))
```
]=]
function colors.black(text: string): string
    return nil :: any
end
--[=[
Turns the provided text green

Usage:

```luau
local colors = require("@std/io/colors")
print(colors.green("this text is hereby colored green"))
```
]=]
function colors.green(text: string): string
    return nil :: any
end
--[=[
Turns the provided text yellow

Usage:

```luau
local colors = require("@std/io/colors")
print(colors.yellow("this text is hereby colored yellow"))
```
]=]
function colors.yellow(text: string): string
    return nil :: any
end
--[=[
Turns the provided text white

Usage:

```luau
local colors = require("@std/io/colors")
print(colors.white("this text is hereby colored white"))
```
]=]
function colors.white(text: string): string
    return nil :: any
end
--[=[
Turns the provided text red

Usage:

```luau
local colors = require("@std/io/colors")
print(colors.red("this text is hereby colored red"))
```
]=]
function colors.red(text: string): string
    return nil :: any
end
colors.bold = {}

--[=[
Turns the provided text bold white

Usage:

```luau
local colors = require("@std/io/colors")
print(colors.bold.white("this is now bold white"))
```
]=]
function colors.bold.white(text: string): string
    return nil :: any
end
--[=[
Turns the provided text bold magenta

Usage:

```luau
local colors = require("@std/io/colors")
print(colors.bold.magenta("this is now bold magenta"))
```
]=]
function colors.bold.magenta(text: string): string
    return nil :: any
end
--[=[
Turns the provided text bold black

Usage:

```luau
local colors = require("@std/io/colors")
print(colors.bold.black("this is now bold black"))
```
]=]
function colors.bold.black(text: string): string
    return nil :: any
end
--[=[
Turns the provided text bold green

Usage:

```luau
local colors = require("@std/io/colors")
print(colors.bold.green("this is now bold green"))
```
]=]
function colors.bold.green(text: string): string
    return nil :: any
end
--[=[
Turns the provided text bold cyan

Usage:

```luau
local colors = require("@std/io/colors")
print(colors.bold.cyan("this is now bold cyan"))
```
]=]
function colors.bold.cyan(text: string): string
    return nil :: any
end
--[=[
Turns the provided text bold yellow

Usage:

```luau
local colors = require("@std/io/colors")
print(colors.bold.yellow("this is now bold yellow"))
```
]=]
function colors.bold.yellow(text: string): string
    return nil :: any
end
--[=[
Turns the provided text bold blue

Usage:

```luau
local colors = require("@std/io/colors")
print(colors.bold.blue("this is now bold blue"))
```
]=]
function colors.bold.blue(text: string): string
    return nil :: any
end
--[=[
Turns the provided text bold red

Usage:

```luau
local colors = require("@std/io/colors")
print(colors.bold.red("this is now bold red"))
```
]=]
function colors.bold.red(text: string): string
    return nil :: any
end


--[=[
An assorted collection of ANSI color codes to help you colorize your text however you want!!

Don't forget to finish off your colorized strings with a `RESET` :p

Usage:

```luau
local colors = require("@std/io/colors")
print(colors.bold.red("this is now bold red"))
```
]=]
colors.codes = {
	RESET = "\x1b[0m",
	BLACK = "\x1b[30m",
	RED = "\x1b[31m",
	GREEN = "\x1b[32m",
	YELLOW = "\x1b[33m",
	BLUE = "\x1b[34m",
	MAGENTA = "\x1b[35m",
	CYAN = "\x1b[36m",
	WHITE = "\x1b[37m",
	BOLD_BLACK = "\x1b[1;30m",
	BOLD_RED = "\x1b[1;31m",
	BOLD_GREEN = "\x1b[1;32m",
	BOLD_YELLOW = "\x1b[1;33m",
	BOLD_BLUE = "\x1b[1;34m",
	BOLD_MAGENTA = "\x1b[1;35m",
	BOLD_CYAN = "\x1b[1;36m",
	BOLD_WHITE = "\x1b[1;37m",
	BRIGHT_BLACK = "\x1b[90m",
	BRIGHT_RED = "\x1b[91m",
	BRIGHT_GREEN = "\x1b[92m",
	BRIGHT_YELLOW = "\x1b[93m",
	BRIGHT_BLUE = "\x1b[94m",
	BRIGHT_MAGENTA = "\x1b[95m",
	BRIGHT_CYAN = "\x1b[96m",
	BRIGHT_WHITE = "\x1b[97m",
	BLACK_BG = "\x1b[40m",
	RED_BG = "\x1b[41m",
	GREEN_BG = "\x1b[42m",
	YELLOW_BG = "\x1b[43m",
	BLUE_BG = "\x1b[44m",
	MAGENTA_BG = "\x1b[45m",
	CYAN_BG = "\x1b[46m",
	WHITE_BG = "\x1b[47m",
	BRIGHT_BLACK_BG = "\x1b[100m",
	BRIGHT_RED_BG = "\x1b[101m",
	BRIGHT_GREEN_BG = "\x1b[102m",
	BRIGHT_YELLOW_BG = "\x1b[103m",
	BRIGHT_BLUE_BG = "\x1b[104m",
	BRIGHT_MAGENTA_BG = "\x1b[105m",
	BRIGHT_CYAN_BG = "\x1b[106m",
	BRIGHT_WHITE_BG = "\x1b[107m",
    BOLD = "\x1b[1m",
    DIM = "\x1b[2m",
}
return colors