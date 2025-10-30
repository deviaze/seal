local fs = require("@std/fs")
local env = require("@std/env")
local err = require("@std/err")
local str = require("@std/str")
local colors = require("@std/colors")
local process = require("@std/process")
local format = require("@std/io/format")
local standalone = require("@interop/standalone")

type Result<T> = err.Result<T>

-- TODO: Add generic parameters to everything once <<>> comes out
-- and recursive type restriction is fixed

export type args = {
    parse: (program: string, tagline: string, info: ProgramInfo?) -> {
        simple: (self: any, ...Arg) -> Parsed,
        commands: (self: any, ...Command) -> Parsed,
    },
    positional: (name: string, help: string) -> Positional,
    named: (name: string, help: string) -> Named,
    command: (name: string, help: string) -> Command,
    flag: (name: string, help: string) -> Flag,
    list: (name: string, help: string) -> ArgList,
    default: (...Arg) -> Command,
    raw: () -> { string },
}

export type ProgramInfo = {
    examples: { string }?,
    description: string?,
    footer: string?
}

export type Command = {
    name: string,
    is: "Command",
    help: string,
    _args: { Arg },
    args: (self: Command, ...Arg) -> Command,
    _aliases: { [string]: true? },
    aliases: (self: Command, ...string) -> Command,
}

export type ArgList = {
    name: string,
    is: "ArgList",
    help: string,
    values: { string }?,
}

export type Validator = (arg: string) -> any | error

export type Arg = 
    | Positional
    | Flag
    | Named
    | ArgList

export type Positional = {
    name: string,
    is: "Positional",
    help: string,
    _default: any,
    default: (any) -> Positional,
    _optional: boolean,
    optional: (self: Positional) -> Positional,
    _validator: Validator?,
    validate: (self: Positional, validator: Validator) -> Positional,
    value: any,
}

export type Flag = {
    name: string,
    is: "Flag",
    help: string,
    _aliases: { [string]: true? },
    aliases: (self: Flag, ...string) -> Flag,
    _default: boolean?,
    default: (self: Flag, boolean) -> Flag,
    value: boolean,
}

export type Named = {
    name: string,
    is: "Named",
    help: string,
    _default: any,
    default: (any) -> Named,
    _aliases: { [string]: true? },
    aliases: (self: Named, ...string) -> Named,
    _required: boolean,
    required: (self: Named) -> Named,
    _validator: Validator?,
    validate: (self: Named, validator: Validator) -> Named,
    value: any,
}

local args = {}

local n5 = "\n     "

local function df(obj: unknown): string
    return if obj == nil then "nil" else format.debug(obj)
end

local function check_name(name: string, is: "Positional" | "Named" | "Flag" | "ArgsList" | "Command", function_name: string): Result<string>
    local heading = `{function_name}:{n5}`
    if typeof(name) ~= "string" then
        return err.wrap(`{heading} expected name to be a string, got: {df(name)}`)
    end

    if name == "--help" then
        return err.wrap(`{heading} argument name '--help' is reserved for displaying help`)
    elseif name == "--commands" then
        return err.wrap(`{heading} sorry, argument name '--commands' is reserved for displaying the command list`)
    end

    if is == "Named" or is == "Flag" then
        if string.find(name, " ", 1, true) then
            return err.wrap(`{heading} please no whitespace in {is} arg name (got '{name}'); makes it incredibly annoying to parse`)
        end
        if not str.startswith(name, "--") then
            return err.wrap(
                if str.startswith(name, "-") then
                    `{heading} single dash '-' reserved for alias names, {is} argument name should start with two dashes '--'`
                else
                    `{heading} {is} name should start with '--'; try '--{name}'`
            )
        end
    elseif is == "Positional" and str.startswith(name, "-") then
        return err.wrap(`{heading} positional argument name should not start with '--' or '-', got: {name}`)
    elseif is == "ArgsList" then
        name = str.trimfront(name, "%.%.%.")
    end

    return name
end

local function check_help(help: string, function_name: string): error?
    if typeof(help) ~= "string" then
        return err.wrap(`{function_name}:{n5} expected help message to be a string, got: {df(help)}`)
    end
    return nil
end

local function check_alias(alias: string, index: number, function_name: string): Result<string>
    if typeof(alias) ~= "string" then
        return err.wrap(`{function_name}:{n5} expected alias {index} to be a string, got: {df(alias)}`)
    end
    if string.find(alias, " ", 1, true) then
        return err.wrap(`{function_name}:{n5} please no whitespace in alias name (got '{alias}'); makes it incredibly annoying to parse`)
    end
    if alias == "-h" then
        return err.wrap(`{function_name}:{n5} alias '-h' is reserved for displaying help`)
    end
    if not str.startswith(alias, "-") then
        return err.wrap(`{function_name}:{n5} alias name should start with '-' (got '{alias}')`)
    end
    return alias
end

function args.positional(name: string, help: string): Result<Positional>
    -- __index optimizations really dont matter for this lib you'll only use it once
    local function_name = "args.positional(name: string, help: string)"

    local checked_name = check_name(name, "Positional", function_name)
    if typeof(checked_name) == "error" then
        return checked_name
    end

    local help_err = check_help(help, function_name)
    if typeof(help_err) == "error" then
        return help_err
    end

    return {
        name = checked_name,
        help = help,
        is = "Positional",
        _default = nil,
        default = function(self: Positional, value: unknown): Positional
            self._default = value
            return self
        end,
        _optional = false,
        optional = function(self: Positional): Positional
            self._optional = true
            return self
        end,
        _validator = nil,
        validate = function(self: Positional, validator: (string) -> any | error): Positional
            self._validator = validator
            return self
        end
    }
end

function args.named(name: string, help: string): Result<Named>
    local function_name = "args.named(name: string, help: string)"

    local checked_name = check_name(name, "Named", function_name)
    if typeof(checked_name) == "error" then
        return checked_name
    end

    local help_err = check_help(help, function_name)
    if typeof(help_err) == "error" then
        return help_err
    end

    return {
        name = checked_name,
        help = help,
        is = "Named",
        default = function(self: Named, value: unknown): Named
            self._default = value
            return self
        end,
        _validator = nil,
        validate = function(self: Named, validator: (string) -> any | error): Named
            self._validator = validator
            return self
        end,
        _required = false,
        required = function(self: Named)
            self._required = true
            return self
        end,
        _aliases = {},
        aliases = function(self: Named, ...: string)
            for index, alias in {...} do
                ecall(function() 
                    return check_alias(alias, index, function_name) 
                end)()
                self._aliases[alias] = true
            end
            return self
        end,
    }
end

function args.flag(name: string, help: string): Result<Flag>
    local function_name = "args.flag(name: string, help: string)"

    local checked_name = check_name(name, "Flag", function_name)
    if typeof(checked_name) == "error" then
        return checked_name
    end

    local help_err = check_help(help, function_name)
    if typeof(help_err) == "error" then
        return help_err
    end

    return {
        name = checked_name,
        help = help,
        is = "Flag",
        _aliases = {},
        aliases = function(self: Flag, ...: string)
            for index, alias in {...} do
                local checked_alias = ecall(function()
                    return check_alias(alias, index, function_name)
                end)()
                self._aliases[checked_alias] = true
            end
            return self
        end,
        _default = false,
        default = function(self: Flag, b: boolean)
            self._default = b
            return self
        end,
        value = false,
    }
end

local function special_flag_help(): Flag
    return {
        name = "--help",
        help = "display this help message",
        is = "Flag",
        _aliases = { ["-h"] = true },
        aliases = function(self: Flag, ...: string) return self end,
        value = false,
        _default = false,
        default = function(self: Flag, b: boolean) return self end,
    }
end

local function special_flag_commands(): Flag
    return {
        name = "--commands",
        help = "display a list of all top-level commands",
        is = "Flag",
        _aliases = {},
        aliases = function(self: Flag, ...: string) return self end,
        value = false,
        _default = false,
        default = function(self: Flag, b: boolean) return self end,
    }
end

function args.list(name: string, help: string): Result<ArgList>
    local function_name = "args.list(name: string, help: string)"

    local checked_name = check_name(name, "ArgsList", function_name)
    if typeof(checked_name) == "error" then
        return checked_name
    end

    local help_err = check_help(help, function_name)
    if typeof(help_err) == "error" then
        return help_err
    end

    return {
        name = checked_name,
        help = help,
        is = "ArgList"
    }
end


function args.default(...: Arg): Command
    return {
        name = "default",
        is = "Command",
        help = "",
        _aliases = {},
        aliases = function(self: Command, ...: string)
            for _, alias in {...} do
                self._aliases[alias] = true
            end
            return self
        end,
        _args = {...},
        args = function(self: Command, ...: Arg)
            for _, arg in {...} do
                table.insert(self._args, arg)
            end
            return self
        end
    }
end

function args.command(name: string, help: string): Result<Command>
    local function_name = "args.command(name: string, help: string)"

    local checked_name = check_name(name, "Command", function_name)
    if typeof(checked_name) == "error" then
        return checked_name
    end

    local help_err = check_help(help, function_name)
    if typeof(help_err) == "error" then
        return help_err
    end

    return {
        name = checked_name,
        help = help,
        is = "Command",
        _aliases = {},
        aliases = function(self: Command, ...: string)
            for _, alias in {...} do
                self._aliases[alias] = true
            end
            return self
        end,
        _args = {},
        args = function(self: Command, ...: Arg)
            for _, arg in {...} do
                table.insert(self._args, arg)
            end
            return self
        end
    }
end

local function parse_args(program_name: string, raw_args: { string }, ...: Arg): Result<{ Arg }>
    local parsed: { Arg } = {}
    local function push(arg: Arg)
        table.insert(parsed, arg)
    end

    local expected_args: { Arg } = {...}
    local function pop_front<T>(t: {T}): T?
        local res = table.remove(t, 1)
        return res
    end

    local expected_index = 0
    local current_raw: string?

    local function consume()
        current_raw = nil
    end

    local finished_positionals = false

    local all_names_and_aliases: { string } = {}
    local function push_name_and_aliases(obj: { name: string, _aliases: { [string]: true? } })
        table.insert(all_names_and_aliases, obj.name)
        for alias, _ in obj._aliases do
            table.insert(all_names_and_aliases, alias)
        end
    end

    while #expected_args > 0 do
        local current_expected = pop_front(expected_args)

        if current_expected == nil then
            break
        end

        expected_index += 1

        local npn = "\n" .. string.rep(" ", #program_name + 2)
        
        if current_expected.is == "Positional" then
            if finished_positionals then
                return err.wrap(`positional argument "{current_expected.name}" should not come after non-positional arguments`)
            end
            if current_raw == nil then
                current_raw = pop_front(raw_args)
            end
            if current_raw == nil then
                if current_expected._default then
                    current_expected.value = current_expected._default
                elseif current_expected._optional == true then
                    continue
                else
                    return err.wrap(`{program_name}: missing required positional argument "{current_expected.name}" ({expected_index})\z
                    {npn}{current_expected.name}: {current_expected.help}`)
                end
            elseif current_expected._validator then
                local result = current_expected._validator(current_raw)
                if typeof(result) == "error" then
                    return err.wrap(`{current_expected.name}: {format.uncolor(tostring(result))} (got "{current_raw}")`)
                end
                current_expected.value = result
            else
                current_expected.value = current_raw
            end
            push(current_expected)
            consume()
        elseif current_expected.is == "Named" then
            push_name_and_aliases(current_expected)
            finished_positionals = true

            -- go through every remaining arg and see if we can find --name=value or --name value or -n value
            local found_value: string?
            local indices_to_remove: { number } = {}
            local function push_indices_to_remove(...: number) 
                for _, index in {...} do table.insert(indices_to_remove, index) end 
            end
            for index, arg in raw_args do
                if str.startswith(arg, current_expected.name) then
                    if string.find(arg, "=", 1, true) then
                        -- --name=value
                        -- or --name="value with = signs"
                        local splits = str.split(arg, "=")
                        pop_front(splits)
                        found_value = table.concat(splits)
                        push_indices_to_remove(index)
                    else
                        -- user might have forgotten the = sign and is using
                        -- whitespace to separate
                        -- --name value
                        local next_arg = raw_args[index + 1]
                        found_value = next_arg
                        push_indices_to_remove(index, index + 1)
                    end
                elseif current_expected._aliases[arg] then
                    -- -n value
                    local next_arg = raw_args[index + 1]
                    found_value = next_arg
                    push_indices_to_remove(index, index + 1)
                else
                    continue
                end
            end
            -- remove indices from raw_args from back to front so we dont misalign while removing
            for i = #raw_args, 1, -1 do
                if table.find(indices_to_remove, i) then
                    table.remove(raw_args, i)
                end
            end

            if not found_value then
                if current_expected._default then
                    found_value = current_expected._default
                elseif current_expected._required then
                    return err.wrap(`{program_name}: missing required named argument "{current_expected.name}"\z
                    {npn}{current_expected.name}: {current_expected.help}`)
                else
                    found_value = nil -- named arguments should not be required by default
                end
            end
            
            current_expected.value = found_value
            push(current_expected)
        elseif current_expected.is == "Flag" then
            push_name_and_aliases(current_expected)
            finished_positionals = true
            local found = false
            for index, arg in raw_args do
                if arg == current_expected.name or current_expected._aliases[arg] then
                    found = true
                    table.remove(raw_args, index)
                    break
                end
            end
            if found then
                current_expected.value = true
                push(current_expected)
            end
        elseif current_expected.is == "ArgList" then
            -- go through all remaining args and keep adding until we see something
            -- that looks like a named arg
            local values: { string } = {}
            while #raw_args > 0 do
                if not str.starts(raw_args[1], "-", "--") then
                    current_raw = pop_front(raw_args)
                    table.insert(values, current_raw :: string)
                    consume()
                else
                    break
                end
            end
            current_expected.values = values
            push(current_expected)
        end
    end

    -- check to make sure user didn't accidentally pass extra args or an unknown named parameter
    if #raw_args > 0 then
        return err.wrap(`{program_name}: got unexpected extra arguments: {table.concat(raw_args, " ")}`)
    end

    return parsed
end

local function help_for_command(command: Command, choose: "simple" | "all" | string)
    type HelpAssoc = {
        is: string,
        names: { string },
        values: { string },
    }
    local function assoc(is: string): HelpAssoc
        return { is = is, names = {}, values = {} }
    end

    local positionals = assoc("Positional Arguments")
    local nameds = assoc("Named Arguments")
    local flags = assoc("Flags")

    local function push(t: HelpAssoc, arg: Arg)
        if arg.is == "Flag" or arg.is == "Named" then
            local parsed_name = {}
            table.insert(parsed_name, arg.name)
            for alias, _ in arg._aliases do
                table.insert(parsed_name, alias)
            end
            table.insert(t.names, table.concat(parsed_name, ", "))
        else
            table.insert(t.names,  arg.name)
        end
        local parsed_help = arg.help do
            local some_default = (arg :: any)._default
            if some_default then
                parsed_help ..= ` (default: {format(some_default)})`
            end
            local is_optional = (arg :: any)._optional
            if is_optional then
                parsed_help ..= " (optional)"
            end
        end
        table.insert(t.values, parsed_help)
    end

    if choose == "simple" or (choose == "all" and command.name == "default") then
        push(flags, special_flag_help())
        if choose == "all" and command.name == "default" then
            push(flags, special_flag_commands())
        end
    end

    local list_tail: ArgList?

    for _, arg in command._args do
        if arg.is == "Positional" then
            push(positionals, arg)
        elseif arg.is == "Flag" then
            push(flags, arg)
        elseif arg.is == "Named" then
            push(nameds, arg)
        elseif arg.is == "ArgList" then
            list_tail = arg
        end
    end

    local front_padding = if choose ~= "simple" then "  " else ""

    local result = {}
    if command.name ~= "default" then
        local command_name = {} do
            table.insert(command_name, command.name)
            for alias, _ in command._aliases do
                table.insert(command_name, alias)
            end
        end
        table.insert(result, front_padding .. colors.bold.yellow(table.concat(command_name, ", ")))
        table.insert(result, string.rep(front_padding, 2) .. command.help .. "\n")
    end

    local function push_help(...: HelpAssoc)
        local function get_max_name_len(...: HelpAssoc): number
            local max_len = 0
            for _, assoc in {...} do
                if #assoc.names == 0 then continue end
                for _, s in assoc.names do
                    if #s > max_len then
                        max_len = #s
                    end
                end
            end
            return max_len
        end
        local max_names_len = get_max_name_len(...)
        local doubled_front_padding = string.rep(front_padding, if command.name == "default" then 1 else 2)
        for _, assoc in {...} do
            if #assoc.names == 0 then continue end
            local argument_type = if choose == "simple" then colors.style.underline(assoc.is) else assoc.is
            table.insert(result, doubled_front_padding .. argument_type .. ":")
            for index, name in assoc.names do
                local help = assoc.values[index]
                local padding = math.max(4, (max_names_len - #name) + 4)
                local simple_trailing_newline = if index == #assoc.names and not list_tail then "\n" else ""
                table.insert(result, `{doubled_front_padding}  {name}{string.rep(" ", padding)}{help}{simple_trailing_newline}`)
            end
        end

        if list_tail then
            local name = "..." .. str.trimfront(list_tail.name, "%.%.%.")
            local help = list_tail.help
            local argument_type = if choose == "simple" then colors.style.underline("List") else "List"
            table.insert(result, doubled_front_padding .. argument_type .. ":")
            local padding = math.max(4, (max_names_len - #name) + 4)
            table.insert(result, `{doubled_front_padding}  {name}{string.rep(" ", padding)}{help}\n`)
        end
    end

    push_help(positionals, nameds, flags)

    return table.concat(result, "\n")
end

local function generate_help(
    program: string, 
    tagline: string,
    info: ProgramInfo,
    choose: "all" | "simple" | string,
    ...: Command
): string
    local result: { string } = {}
    local function push(line: string, newlines: number?)
        newlines = newlines or 1
        table.insert(result, line .. string.rep("\n", newlines))
    end

    if choose == "all" or choose == "simple" then
        if info.description then
            push(`{colors.bold.blue(program)} {colors.style.dim("-")} {tagline}`, 2)
            push(info.description, 2)
        else
            push(colors.bold.blue(program))
            push(tagline, 2)
        end

        push(colors.style.underline(colors.bold.white("Usage")))
        
        local program_name =
            if standalone.check(env.executable_path) then
                `{colors.bold.blue(fs.path.child(env.executable_path) :: string)}`
            else
                `{colors.bold.blue("seal")} path/to/{fs.path.child(script.entry_path)}`
        
        push(`  {program_name} {
            if choose == "simple" then
                "[arguments]"
            else
                "[command] [arguments]"
        }`, 2)

        if info.examples then
            push(colors.style.underline(colors.bold.green(if #info.examples == 1 then "Example" else "Examples")))
            for _, example in info.examples do
                push(`  {program_name} {example}`)
            end
        end

        push("", 1)
    end

    if choose ~= "simple" then
        push(colors.style.underline(colors.bold.yellow("Commands")))
    end

    for _, arg in {...} do
        if arg.is == "Command" then
            if choose == "all" or choose == "simple" or choose == arg.name then
                push(help_for_command(arg, choose))
            end
        end
    end

    if info.footer then
        push(info.footer)
    end

    return table.concat(result)
end

local Parsed = {}
type Parsed = typeof(Parsed.from(...))

local ParsedImpl = {}
ParsedImpl.__index = ParsedImpl

function Parsed.from(
    config: {
        program: string,
        tagline: string,
        choose: string,
        command: Command,
        commands: { Command },
        info: ProgramInfo,
    },
    parsed: { Arg }
)
    local flags: { [string]: true } = {} do
        for _, arg in parsed do
            if arg.is == "Flag" then
                flags[arg.name] = true
            end
        end
    end

    local self = {
        command = config.command.name,
        config = config,
        flags = flags,
        args = parsed,
    }

    return setmetatable(self, ParsedImpl)
end

function Parsed.get<T>(self: Parsed, name: string, default: T): Result<T?>
    local found_names: { string } = {}
    for _, arg in self.args do
        local normalized_name = str.trimfront(arg.name, "%-")
        table.insert(found_names, normalized_name)
        if name == arg.name or name == normalized_name then
            if arg.is == "ArgList" then
                return arg.values
            elseif arg.value then
                return arg.value
            elseif arg._default then
                return arg._default
            elseif default then
                return default
            else
                return nil
            end
        end
    end
    return err.wrap(`got unexpected argument name '{name}'; expected argument names (normalized): {table.concat(found_names, ", ")}`)
end

function Parsed.expect<T>(self: Parsed, name: string, assertion: string?): Result<T>
    local found_names: { string } = {}
    for _, arg in self.args do
        local normalized_name = str.trimfront(arg.name, "-")
        table.insert(found_names, normalized_name)
        if name == arg.name or name == normalized_name then
            if arg.is == "ArgList" then
                return arg.values
            elseif arg.value then
                return arg.value
            elseif arg._default then
                return arg._default
            else
                return err.wrap(
                    if assertion then
                        assertion
                    else
                        `argument '{name}' assertion failed (no value)`
                )
            end
        end
    end
    return err.wrap(`got unexpected argument name '{name}'; expected argument names (normalized): {table.concat(found_names, ", ")}`)
end

function Parsed.help(self: Parsed): string
    local config = self.config
    return generate_help(
        config.program,
        config.tagline, 
        config.info, 
        config.choose, 
        table.unpack(config.commands)
    )
end

ParsedImpl.get = ecall(Parsed.get)
ParsedImpl.expect = ecall(Parsed.expect)
ParsedImpl.help = ecall(Parsed.help)

function args.parse(program: string, tagline: string, info: ProgramInfo?)
    local function show_help(choose: string?, message: string | false, ...: Command): never
        local help_message = generate_help(program, tagline, info or {}, choose or "all", ...)
        if message then
            print(colors.red(`{program}: {message}, displaying --help\n`))
        end
        print(help_message)
        return process.exit(if typeof(message) == "string" then 1 else 0)
    end

    return {
        simple = ecall(function(self: any, ...: Arg | Result<Arg>): Result<Parsed>
            local commands = args.default(...)
            local first_arg = env.args[1]
            if first_arg == "--help" or first_arg == "-h" then
                show_help("simple", false, commands)
            end

            local parsed_result = parse_args(program, table.clone(env.args), ...)
            if typeof(parsed_result) == "error" then
                print(tostring(parsed_result))
                show_help("simple", "unable to parse arguments", commands)
            else
                return Parsed.from({
                    program = program,
                    tagline = tagline,
                    choose = "simple",
                    command = commands,
                    commands = { commands },
                    info = info or {},
                }, parsed_result)
            end
            error("unreachable")
        end),
        commands = ecall(function(self: any, ...: Command): Result<Parsed>
            local first_arg = env.args[1]
            if (first_arg :: string?) == nil then
                first_arg = "default" 
            end

            if first_arg == "--help" or first_arg == "-h" then
                show_help("all", false, ...)
            elseif first_arg == "--commands" then
                type Assocs = {
                    names: { string },
                    helps: { string },
                }
                local assocs: Assocs = { names = {}, helps = {} }
                for _, command in {...} do
                    local parsed_command_name = {}
                    table.insert(parsed_command_name, command.name)
                    for alias, _ in command._aliases do
                        table.insert(parsed_command_name, alias)
                    end
                    table.insert(assocs.names, table.concat(parsed_command_name, ", "))
                    table.insert(assocs.helps, command.help)
                end

                local result: { string } = {}
                local function push(s: string, newlines: number?)
                    newlines = newlines or 1
                    table.insert(result, s .. string.rep("\n", newlines))
                end

                push(colors.style.underline("Commands"))
                local max_names_len = 0 do
                    for _, name in assocs.names do
                        if #name > max_names_len then
                            max_names_len = #name
                        end
                    end
                end
                for index, command_name in assocs.names do
                    local help = assocs.helps[index]
                    local padding = string.rep(" ", (max_names_len - #command_name) + 4)
                    push(`  {command_name}{padding}{help}`)
                end
                
                print(table.concat(result))
                process.exit(1)
            end

            local command_names_and_aliases: { [string]: Command } = {} do
                for _, command in {...} do
                    if command_names_and_aliases[command.name] then
                        return err.wrap(`duplicate command name ({command.name})`)
                    end
                    command_names_and_aliases[command.name] = command
                    for alias, _ in command._aliases do
                        if command_names_and_aliases[alias] then
                            return err.wrap(`duplicate alias name (alias {alias} of command {command.name})`)
                        end
                        command_names_and_aliases[alias] = command
                    end
                end
            end

            local choose =
                if first_arg == "default" or command_names_and_aliases[first_arg] == nil then
                    "all"
                else
                    first_arg

            local parsed
            if command_names_and_aliases[first_arg] then
                local cloned = table.clone(env.args)
                table.remove(cloned, 1) -- pop command name (first arg)
                local result = parse_args(program, cloned, table.unpack(command_names_and_aliases[first_arg]._args))
                if typeof(result) == "error" then
                    -- return err.wrap(`can't parse args due to reason {result}`)
                    print(tostring(result))
                    show_help(first_arg, "unable to parse arguments", ...)
                end

                parsed = result
            elseif command_names_and_aliases["default"] then
                -- assume first arg is an argument for the default command and not a command of itself
                local result = parse_args(program, table.clone(env.args), table.unpack(command_names_and_aliases["default"]._args))
                if typeof(result) == "error" then
                    -- return err.wrap(`can't parse default command args due to reason {result}`)
                    print(tostring(result))
                    show_help("all", "unable to parse arguments for default command", ...)
                end

                parsed = result
            else
                if first_arg == "default" then
                    show_help("all", "no command specified", ...)
                else
                    show_help(choose, `unknown command: {first_arg}\n{string.rep(" ", #program + 2)}pass --commands for a list of valid commands`, ...)
                end
            end

            assert(parsed ~= nil, "parsed should not be nil here")

            parsed = Parsed.from({
                program = program,
                tagline = tagline,
                choose = choose,
                command = command_names_and_aliases[first_arg] or command_names_and_aliases["default"],
                commands = {...},
                info = info or {},
            }, parsed)

            return parsed
        end)
    }
end

return args :: args
