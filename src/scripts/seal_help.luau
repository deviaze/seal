local colors = require("@std/colors")
local dim = colors.style.dim

local help = {}

local function format_lines(...: string): string
    local result = ""
    local lines_to_merge = { ... }
    for index, line in lines_to_merge do
        result ..= if index ~= #lines_to_merge then line .. "\n" else line
    end
    return result
end

type ColorCode = keyof<typeof(colors.codes)>

local function command(name: string, alt: string?, color: ColorCode | string): string
    local color_code = (colors.codes :: any)[color]
    return 
        color_code .. name .. colors.codes.RESET 
        .. if alt then dim " | " .. color_code .. alt .. colors.codes.RESET else ""
end

local function req(arg: string, ...: string): string
    local res = dim "<" .. colors.style.bold(arg)
    for _, alternative in { ... } do
        res ..= dim " | " .. colors.style.bold(alternative)
    end
    res ..= dim ">"
    return res
end

local function opt(arg: string, ...: string): string
    local res = dim "[" .. colors.style.bold(arg)
    for _, alternative in { ... } do
        res ..= dim " | " .. colors.style.bold(alternative)
    end
    res ..= dim "]"
    return res
end

local function code(c: string): string
    return dim "`" .. c .. dim "`"
end

local SEAL_VERSION = _SEAL_VERSION
local UNDERLINE = "\27[4m"
local DIM_DASH = dim "-"

local TAGLINE = format_lines(
    `{colors.bold.blue("seal")} {dim(`({SEAL_VERSION})`)} {colors.bold.white("-")} the cutest runtime for luau`
)

local FAQ = format_lines(
    `{DIM_DASH} To {colors.bold.white "run a file"} with seal, simply run {code "seal ./myfile.luau"}`,
    `{DIM_DASH} To start a {colors.bold.white "new project"} with seal, run {code "seal setup"}`,
    `{DIM_DASH} To view standard library documentation, check out {code ".seal/typedefs"}`
)

local END_LINE = format_lines(
    "",
    `To submit a feature request or report a bug: {UNDERLINE}https://github.com/deviaze/seal{colors.codes.RESET}`,
    "Thank you for using seal :)"
)

function help.default()
    return format_lines(
        TAGLINE,
        FAQ,
        "",
        colors.bold.white("Usage:") .. dim(" <angled> = required, [square] = optional"),
        `  seal {req("command", "filename")} {opt "...args"}`,
        "",
        colors.bold.yellow "Commands:" .. dim " (use seal help <command> for detailed info about a specific command)",
        `  { command("seal", nil, "BOLD_BLUE")}  {req "filename.luau"} {opt "...args"} {DIM_DASH} run a file {dim "(must end with .luau or be directory w/ init.luau)"}.`,
        `  { command("setup", "s", "BOLD_CYAN")} {DIM_DASH} a new project in your current directory; sets up all you need to get started with seal.`,
        `  { command("eval ", "e", "BOLD_RED")}  '{req "src"}'    {DIM_DASH} evaluate luau code from a string, right in your terminal.`,
        `  { command("run  ", "r", "BOLD_GREEN")}  {opt "...args"}  {DIM_DASH} run the {colors.bold.white("current project")} at its entrypoint; similar to { code "cargo run"} in Rust.`,
        `  { command("test ", "t", "BRIGHT_GREEN")}  {opt "...args"}  {DIM_DASH} runs 'test_path' from your { code ".seal/config.luau"}.`,
        `  { command("help ", "h", "BOLD_WHITE")}  {opt "command"}  {DIM_DASH} display help, of a specific command if specified.`,
        "",
        colors.bold.magenta("Flags:"),
        `  {dim "--help | -h"} - same as seal help; can be used like seal {dim "--help"} or seal <command> {dim "--help"}`,
        END_LINE
    )
end

function help.eval()
    return format_lines(
        TAGLINE,
        colors.bold.white("Usage:") .. dim(" <angled> = required, [square] = optional"),
        `  {colors.bold.blue("seal")} {colors.bold.red("eval")} '{req "src"}' {DIM_DASH} evaluate luau code from a string src`,
        colors.bold.white("Example:"),
        `  seal eval 'p(http.get("http://idk.com/api/hi"):unwrap_json())'`,
        colors.bold.white("Info:"),
        `  {DIM_DASH} evaluate luau code from a string with common libs already loaded in for convenience.`,
        `  libraries loaded in by default:`,
        `    {dim "@std/"}fs {DIM_DASH} filesystem stuff, use fs.readfile/writefile directly.`,
        `    {dim "@std/"}process {DIM_DASH} process.run, process.spawn, etc.`,
        `    {dim "@std/net/"}http {DIM_DASH} send http requests with http.get, http.request, etc.`,
        END_LINE
    )
end

function help.run()
    return format_lines(
        TAGLINE,
        colors.bold.white("Usage:") .. dim(" <angled> = required, [square] = optional"),
        `  {colors.bold.blue("seal")} {colors.bold.green("run")} {opt "...args"}` .. `{DIM_DASH} run the current project at its entrypoint (default ./src/main.luau)`,
        colors.bold.white("Info:"),
        `  To configure the project's entrypoint, edit the 'entry_path' field of {code "./.seal/config.luau"}.`,
        `  If you don't have a .seal folder yet, run {code "seal setup"} to generate one in your current directory.`,
        `  For more info on config.luau, run {code "seal help config"}`,
        END_LINE
    )
end

function help.test()
    return format_lines(
        TAGLINE,
        colors.bold.white("Usage:") .. dim(" <angled> = required, [square] = optional"),
        `  {colors.bold.blue("seal")} {colors.bold.green("test")} {opt "...args"} {DIM_DASH} test the current project at its test_path`,
        colors.bold.white("Info:"),
        `  Configure your test_path in ./.seal/config.luau`,
        END_LINE
    )
end

function help.setup()
    return format_lines(
        TAGLINE,
        colors.bold.white("Usage:") .. dim(" <angled> = required, [square] = optional"),
        `  {colors.bold.blue("seal")} {colors.bold.magenta("setup")} {opt("project", "script", "custom")}`,
        `  {colors.bold.blue("seal")} {colors.bold.magenta("project")} {DIM_DASH} start a new standalone seal project in your current directory`,
        `  {colors.bold.blue("seal")} {colors.bold.magenta("script")}  {DIM_DASH} use seal for custom scripts in an existing codebase`,
        `  {colors.bold.blue("seal")} {colors.bold.magenta("custom")}  {DIM_DASH} start a new codebase with custom options (interactive!)`,
        `       {opt("--codebase project | script")} {opt("--editor vs(code) | zed")} {opt("--typedefs gen | link")} {opt("--git true | false")}`,
        `  {colors.bold.blue("seal")} {req("sp", "ss", "sc")} {DIM_DASH} short forms for seal project, script, and custom`,
        colors.bold.white("Options:"),
        `  {DIM_DASH} seal setup configures the following for you depending on your project type, preserving any existing settings:`,
        `  {colors.bold.blue("Projects")}`,
        `    {dim ">"} Projects are self-contained and typically are their own codebases.`,
        `    {dim ">"} seal generates new typedefs for each project for ease of portability and compilation.`,
        `    {DIM_DASH} {colors.bold.cyan(".luaurc")} {DIM_DASH} configuration for the luau language, including aliases, strict mode, etc.`,
        `    {DIM_DASH} {colors.bold.white(".vscode/settings.json")} {DIM_DASH} adds seal's globals.d.luau to definition files`,
        `    {DIM_DASH} {colors.bold.white("./src/main.luau")} {DIM_DASH} a place to get started on your code (run it with {code "seal run"})`,
        `    {DIM_DASH} {colors.bold.blue(".seal")} {DIM_DASH} a seal folder`,
        `      {DIM_DASH} {colors.bold.white("typedefs")} {DIM_DASH} type definitions and documentation for seal and its stdlibs!`,
        `      {DIM_DASH} {colors.bold.blue("config.luau")} {DIM_DASH} configure the entrypoint of your project, test path, and other settings.`,
        `      {DIM_DASH} {colors.bold.magenta("guided_tour.luau")} {DIM_DASH} a cookbook for the most common things you might want to accomplish with seal!`,
        `      {DIM_DASH} {colors.bold.yellow("@extra")} {DIM_DASH} common libraries you might want that aren't covered in the stdlib; you can modify these.`,
        `    {DIM_DASH} {colors.bold.red(".git")} {DIM_DASH} creates git repository for you if one doesn't exist.`,
        `  {colors.bold.blue("Scripts")}`,
        `    {dim ">"} Scripts are for when you want to use seal as an addendum to an existing project.`,
        `    {dim ">"} For script codebases, seal links to a top-level ~/.seal directory instead of generating new typedefs.`,
        `    {DIM_DASH} {colors.bold.cyan(".luaurc")} {DIM_DASH} configuration for the luau language, including aliases, strict mode, etc.`,
        `    {DIM_DASH} {colors.bold.white(".vscode/settings.json")} {DIM_DASH} adds seal's globals.d.luau to definition files`,
        `    {DIM_DASH} {colors.bold.blue(".seal")} {DIM_DASH} a seal folder`,
        `      {DIM_DASH} {colors.bold.blue("config.luau")} {DIM_DASH} configure the entrypoint of your project, test path, and other settings.`,
        `      {DIM_DASH} {colors.bold.blue("script.luau")} {DIM_DASH} an entrypoint for your scripting codebase.`,
        `  {colors.bold.blue("Custom")}`,
        `    {dim ">"} Mix and match settings as you need! Configurable options are:`,
        `    {DIM_DASH} --codebase (project or script)`,
        `    {DIM_DASH} --editor (vscode or code (same as vscode) or zed)`,
        `    {DIM_DASH} --typedefs (gen or link) (do you want to generate new typedefs in this project or link to ~/.seal?)`,
        `    {DIM_DASH} --git (true or false or y or n) (make this a new git repository?)`,
        END_LINE
    )
end

function help.config()
    return format_lines(
        TAGLINE,
        colors.bold.white("Info:"),
        ` {DIM_DASH} found at {code "./.seal/config.luau"}`,
        ` {DIM_DASH} valid fields are located in the SealConfig typedef`,
        END_LINE
    )
end

function help.help()
    return format_lines(
        TAGLINE,
        colors.bold.white("Usage:") .. dim(" <angled> = required, [square] = optional"),
        `  {colors.bold.blue("seal")} {colors.bold.magenta("help")}` .. ` {DIM_DASH} Help`,
        colors.bold.white("Info:"),
        `  {DIM_DASH} pass a command or 'config' as the optional argument for specific info`,
        END_LINE
    )
end

return help