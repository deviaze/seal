local str = require("@std/str")
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

local SEAL_VERSION = str.split(_VERSION, " ")[2]
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
		`  {colors.bold.blue("seal")} {colors.bold.magenta("setup")}` .. ` {DIM_DASH} Create a new project in your current directory, setting up typedefs, config, etc.`,
		colors.bold.white("Info:"),
		`  {colors.blue("seal")} wants to make it easy and painless to start programming in Luau.`,
		`  {DIM_DASH} Want a throwaway script? seal setup. {dim "(you'll probably end up with > 1 file anyway)"}`,
		`  {DIM_DASH} Want an actual project? seal setup.`,
		`  {colors.bold.green("What's included:")}`,
		`  {DIM_DASH} seal setup sets up and configures the following for you, preserving any existing settings:`,
		`    {DIM_DASH} {colors.bold.cyan(".luaurc")} {DIM_DASH} configuration for the luau language, including aliases, strict mode, etc.`,
		`    {DIM_DASH} {colors.bold.white(".vscode/settings.json")} {DIM_DASH} adds seal's globals.d.luau to definition files`,
		`    {DIM_DASH} {colors.bold.white("./src/main.luau")} {DIM_DASH} a place to get started on your code (run it with {code "seal run"})`,
		`    {DIM_DASH} {colors.bold.blue(".seal")} {DIM_DASH} a seal folder`,
		`      {DIM_DASH} {colors.bold.white("typedefs")} {DIM_DASH} type definitions and documentation for seal and its stdlibs!`,
		`      {DIM_DASH} {colors.bold.blue("config.luau")} {DIM_DASH} configure the entrypoint of your project, test path, and other settings.`,
		`      {DIM_DASH} {colors.bold.magenta("guided_tour.luau")} {DIM_DASH} a cookbook for the most common things you might want to accomplish with seal!`,
		`      {DIM_DASH} {colors.bold.yellow("@extra")} {DIM_DASH} common libraries you might want that aren't covered in the stdlib; you can modify these.`,
		`    {DIM_DASH} {colors.bold.red(".git")} {DIM_DASH} creates git repository for you if one doesn't exist.`
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