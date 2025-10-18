local str = require("@std/str")
local err = require("@std/err")
local time = require("@std/time")
local colors = require("@std/colors")
local format = require("@std/io/format")

-- note!! requiring the whole @std/io library here will cause overflow panic
-- cause this is *part* of the @std/io library!!
local input = require("@std/io/input")
local output = require("@std/io/output")

local n5 = "\n     "

local prompt = {}

local function df(v: any)
    return if v == nil then "nil" else format.debug(v)
end

function prompt.validate(message: string, v: (response: string) -> true | string): err.Result<string>
    local function_name = "prompt.validate(message: string, v: (response: string) -> true | string)"

    if typeof(message) ~= "string" then
        return err.wrap(`{function_name}{n5} expected message to be a string, got: {df(message)}`)
    end

    if typeof(v) ~= "function" then
        return err.wrap(`{function_name}{n5} expected validator function, got: {df(v)}`)
    end

    if not string.find(message, ": ", 1, true) then
        message ..= ": "
    end

    local result = input.readline(message)

    if typeof(result) == "string" then
        local _, validation = pcall(v, result)
        if validation == true then
            return result
        elseif typeof(validation) == "string" then
            warn(validation)
            if not str.startswith(message, "Try again?") then
                message = "Try again? " .. message
            end
            return prompt.validate(message, v)
        elseif typeof(validation) == "error" then
            return err.wrap(`{function_name}:{n5} validator function errored at runtime: {tostring(validation)}`)
        else
            return err.wrap(`{function_name}:{n5} validator function returned an invalid type, expected true or a string, got: {df(validation)}`)
        end
    elseif typeof(result) == "interrupt" then
        return err.wrap(`Prompt interrupted with {if result.code == "CtrlC" then "Ctrl-C" else "Ctrl-D"}; use io.input.readline to intercept without erroring`)
    elseif typeof(result) == "error" then
        return err.wrap(`{function_name}: unexpected ReadlineError: {tostring(result)}`)
    end

    error("unreachable")
end

function prompt.pick(message: string, options: { string }, default: number?): err.Result<number>
    local function_name = "prompt.pick(message: string, options: { string }, default: number?)"

    if typeof(message) ~= "string" then
        return err.wrap(`{function_name}{n5} expected message to be a string, got: {df(message)}`)
    end

    if typeof(options) ~= "table" then
        return err.wrap(`{function_name}{n5} expected options to be an array-like table (\{ string \}), got: {df(options)}`)
    end

    if #options == 0 then
        return err.wrap(`{function_name}:{n5} passed options table is empty or isn't an array-like table\ngot options = {format.simple(options)}`)
    end

    if not string.find(message, ": ", 1, true) then
        message ..= ": "
    end

    if default then
        if typeof(default) ~= "number" then
            return err.wrap(`{function_name}:{n5} default value should be nil/unspecified or an integer number, got: {df(default)}`)
        elseif default ~= math.round(default) then
            return err.wrap(`{function_name}:{n5} default value should be an integer number, got a float ({default})`)
        elseif default <= 0 then
            return err.wrap(`{function_name}:{n5} default value should be positive (got {default})`)
        elseif default > #options then
            return err.wrap(`{function_name}:{n5} default value out of range (options table has {#options} value{if #options > 1 then "s" else ""} but default was {default})`)
        end
    end

    local index: number
    
    if input.tty() then
        print(message)

        input.rawmode(true)

        output.write("\27[?25l") -- hide cursor

        local current = default or 1
        for option_index, option in options do -- write all the options down
            output.write(`{if option_index == current then colors.bold.green(">") else " "}  {colors.bold.blue(tostring(option_index))} {option}\r\n`)
        end
        for i = #options, 1, -1 do
            output.write("\27[A")
            if i == current then -- go back to the default
                break 
            end
        end

        local interrupt_message: string?

        for event in input.events(time.milliseconds(20)) do
            if event.is == "Key" then
                if event.key == "Up" and current - 1 > 0 then
                    current -= 1
                    output.write("\r ") -- erase the > at the start of the current line
                    output.write("\27[A") -- go up 1 visually
                    output.write("\r" .. colors.bold.green(">")) -- write a new > at the current line
                elseif event.key == "Down" and current < #options then
                    current += 1
                    output.write("\r ")
                    output.write("\27[B") -- go down 1 visually
                    output.write("\r" .. colors.bold.green(">"))
                elseif event.key == "Enter" then
                    index = current
                    break
                elseif event.key == "c" and event.modifiers.ctrl then
                    interrupt_message = "Prompt interrupted with Ctrl+C"
                    index = current
                    break
                elseif event.key == "d" and event.modifiers.ctrl then
                    interrupt_message = "Prompt interrupted with Ctrl+D"
                    index = current
                    break
                end
            end
        end

        while current < #options do -- go to bottom so we can start erasing rows
            current += 1
            output.write("\27[B")
        end
        for i = 1, (#options - 1) do
            output.write("\27[A")         -- move up one line
            output.write("\27[2K\27[G")   -- erase line and go to start
        end

        output.write("\27[?25h") -- show cursor
        
        input.rawmode(false)

        if interrupt_message then
            return err.wrap(interrupt_message)
        end
    else
        print(message)

        print()

        for option_index, option in options do
            print(`  {option_index}: {option}`)
        end

        print()

        while true do
            local response = str.trimback(input.rawline("Pick an index from the above: "))
            response = tonumber(response)
            if response and response > 0 and response < #options then
                index = response
                break
            else
                print(`Invalid index, try again?`)
            end
        end
    end

    return index
end

return prompt