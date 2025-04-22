--!optimize 2
--!nolint LocalShadow
--[[
    doing highly recursive operations + lots of table iteration is faster directly in luau
    rather than through mlua's interop layer, which, among other things, has additional safety costs
    associated with iterating through tables
]]

local mlua = require("@interop/mlua")
local colors = require("@std/colors")
local output = require("@std/io/output")

local VECTOR_FOUR_WIDE_MODE = (function()
    local s, f = pcall(function()
        local testvec = (vector.create :: any)(1, 2, 3, 4)
        print(testvec.w)
    end)
    return s
end)()

local function get_table_id(t: { [any]: any }): string
    if typeof(t) ~= "table" then
        error("[INTERNAL] output_process_values.get_table_id got passed something that isn't a table")
    end
    local tostringed_t = tostring(t)
    -- remove the `table: ` part of `table: 0x000062524e10a0d8`
    local tableid = string.sub(tostringed_t, 8, #tostringed_t)
    return tableid
end

local IDENTIFIER_PATTERN = "^[%a_][%w_]*$"

local function process_raw_values(
    value: any,
    seen_tables: { [string]: string }?,
    depth: number?
): string
    seen_tables = seen_tables or {}
    local result = ""
    local depth = depth or 0
    local current_indent = string.rep("    ", depth)
    if typeof(value) == "table" then
        local value = value :: { [any]: any }
        local table_id = get_table_id(value)
        local seen_name = seen_tables[table_id]
        if seen_name then
            result ..= "cyclic(" .. seen_name .. ")"
        else
            if depth == 0 then
                seen_tables[table_id] = "self" -- so we can recognize top level cyclic tables
            end
            result ..= "{\n"
            for key, val in value do
                result ..= current_indent
                result ..= "    "
                -- handle key stringification
                if typeof(key) == "string" then
                    -- we can omit brackets if key is an identifier-like key (like "meow1234" which is vs "cats are cool" which isn't)
                    if string.match(key, IDENTIFIER_PATTERN) then
                        result ..= key
                    else
                        result ..= "[\"" .. key .. "\"]"
                    end
                elseif typeof(key) == "number" then
                    if mlua.isint(key) then
                        result ..= "[" .. tostring(key) .. "]"
                    else -- is a LuaNumber
                        if math.round(key) == key then
                            result ..= "[" .. tostring(key) .. ".0]"
                        else
                            result ..= "[" .. tostring(key) .. "]"
                        end
                    end
                elseif typeof(key) == "table" then
                    local key = key :: { [any]: any }
                    local key_tableid = get_table_id(key)
                    local seen_key_name = seen_tables[key_tableid]
                    if seen_key_name then
                        result ..= "[cyclic(" .. seen_key_name .. ")]"
                    else
                        -- according to ariel et al, people probably want to see addresses in debug mode
                        local stringified_tableid = string.gsub(table_id, "0x0*", "")
                        result ..= "[table(0x" .. stringified_tableid .. ")]"
                        -- but pretty print should show full table
                    end
                else
                    result ..= "[" .. process_raw_values(key, seen_tables, depth + 1) .. "]"
                end
                result ..= " = "
                -- handle val stringification
                if typeof(val) == "table" then
                    local val = val :: { [any]: any }
                    local val_tableid = get_table_id(val)
                    local seen_val_name = seen_tables[val_tableid]
                    if seen_val_name then
                        result ..= "cyclic(" .. seen_val_name .. ")"
                    else
                        result ..= process_raw_values(val, seen_tables, depth + 1)
                    end
                elseif typeof(val) == "string" then -- no need for extra recursive call for common case
                    result ..= '"' .. val .. '"'
                else
                    result ..= process_raw_values(val, seen_tables, depth + 1)
                end
                result ..= ",\n"
            end
        end
        result ..= current_indent ..  "}"
    elseif typeof(value) == "string" then
        result ..= if depth == 0 then value else  '"' .. value .. '"'
    elseif typeof(value) == "boolean" then
        result ..= tostring(value)
    elseif typeof(value) == "number" then
        if mlua.isint(value) then
            result ..= tostring(value)
        else
            if math.round(value) == value then
                result ..= tostring(value) .. ".0"
            else
                result ..= tostring(value)
            end
        end
    elseif typeof(value) == "function" then
        local function_id = tostring(value)
        function_id = string.gsub(function_id, "function: 0x0*", "")
        return "function<0x" .. function_id .. ">"
    elseif typeof(value) == "buffer" then
        local buffer_id = tostring(value)
        buffer_id = string.gsub(buffer_id, "buffer: 0x0*", "")
        return "buffer<0x" .. buffer_id .. ">"
    elseif type(value) == "vector" then
        local vec = "vector<" .. value.x .. ", " .. value.y .. ", " .. value.z
        if VECTOR_FOUR_WIDE_MODE then
            vec ..= ", " ..  value.z
        end
        vec ..= ">"
        return vec
    elseif mlua.iserror(value) then
        local stringified_error = tostring(value)
        local errsplit = string.split(stringified_error, "\n")
        local errmsglist = {}
        for _, m in errsplit do
            if string.find(m, "stack traceback") then
                break
            else
                table.insert(errmsglist, m)
            end
        end
        return "error<" .. output.unformat(table.concat(errmsglist)) .. ">"
    elseif type(value) == "userdata" then
        local typeof_value = typeof(value)
        if typeof_value ~= "userdata" then
            return "userdata<" .. typeof_value .. ">"
        else
            local userdata_id = tostring(value)
            userdata_id = string.gsub(userdata_id, "userdata: 0x0*", "")
            return "userdata<0x" .. userdata_id .. ">"
        end
    elseif type(value) == "nil" then
        return "nil"
    elseif type(value) == "thread" then
        local coro_id = tostring(value)
        coro_id = string.gsub(coro_id, "thread: 0x0*", "")
        return "coroutinethread<0x" .. coro_id .. ">"
    else
        return tostring(value)
    end
    return result
end

local RESET = colors.codes.RESET
local BOLD_WHITE = colors.codes.BOLD_WHITE
local RED = colors.codes.RED
local BOLD_RED = colors.codes.BOLD_RED
local CYAN = colors.codes.CYAN
local MAGENTA = colors.codes.MAGENTA
local BOLD_MAGENTA = colors.codes.BOLD_MAGENTA
local BLUE = colors.codes.BLUE
local BOLD_BLUE = colors.codes.BOLD_BLUE
local GREEN = colors.codes.GREEN
local YELLOW = colors.codes.YELLOW
local BOLD_YELLOW = colors.codes.BOLD_YELLOW
local BLACK = colors.codes.BLACK
local BOLD = colors.codes.BOLD
local DIM = colors.codes.DIM

local LEFT_BRACKET = DIM .. "[" .. RESET
local RIGHT_BRACKET = DIM .. "]" .. RESET

local LEFT_ANGLED = BOLD .. DIM .. "<" .. RESET
local RIGHT_ANGLED = BOLD .. DIM .. ">" .. RESET

local ID_CATCHER_PATTERN = "[.]*0x0*([%d%w]+)"

local function stringify_simple_type(value: any): string
    if type(value) == "number" then
        -- mlua Numbers are BOLD_BLUE and Integers are regular BLUE
        if mlua.isint(value) then
            return BLUE .. tostring(value) .. RESET
        else
            return BOLD_BLUE .. tostring(value) .. (if math.round(value) == value then ".0" else "") .. RESET
        end
    elseif type(value) == "string" then
        return GREEN .. '"' .. value .. '"' .. RESET
    elseif type(value) == "boolean" then
        return YELLOW .. tostring(value) .. RESET
    elseif type(value) == "vector" then
        return BOLD_RED .. "vector" .. RESET .. LEFT_ANGLED
               .. BOLD_BLUE .. value.x .. CYAN .. ", "
               .. BOLD_BLUE .. value.y .. CYAN .. ", "
               .. BOLD_BLUE .. value.z .. CYAN ..
               (if VECTOR_FOUR_WIDE_MODE then ", " ..
                  BOLD_BLUE .. value.w
               else "") 
               .. RED .. RIGHT_ANGLED .. RESET
    elseif type(value) == "function" then
        local function_id = string.match(tostring(value), ID_CATCHER_PATTERN)
        local last_five = string.sub(function_id, #function_id - 4, # function_id)
        local first_part = string.sub(function_id, 1, #function_id - 5)
        return BOLD_RED .. "function" .. RESET .. LEFT_ANGLED .. RED .. "0x" ..  first_part .. colors.bold.red(last_five) .. RESET .. RED .. RIGHT_ANGLED
    elseif type(value) == "buffer" then
        local buffer_id = string.match(tostring(value), ID_CATCHER_PATTERN)
        local last_five = string.sub(buffer_id, #buffer_id - 4, # buffer_id)
        local first_part = string.sub(buffer_id, 1, #buffer_id - 5)
        return BOLD_MAGENTA .. "buffer" .. RESET .. LEFT_ANGLED .. MAGENTA .. "0x" .. first_part .. colors.bold.magenta(last_five) .. RESET .. MAGENTA .. RIGHT_ANGLED
    elseif mlua.iserror(value) then
        local stringified_error = tostring(value)
        local errsplit = string.split(stringified_error, "\n")
        local errmsglist = {}
        for _, m in errsplit do
            if string.find(m, "stack traceback") then
                break
            else
                table.insert(errmsglist, m)
            end
        end
        return BOLD_RED .. "error" .. RESET .. RED .. LEFT_ANGLED .. table.concat(errmsglist) .. RIGHT_ANGLED
    elseif type(value) == "thread" then
        local thread_id = string.match(tostring(value), ID_CATCHER_PATTERN)
        return "corothread" .. LEFT_ANGLED .. "0x" .. thread_id .. RIGHT_ANGLED
    elseif type(value) == "table" then -- treat a table as a simple value
        local table_id = string.match(tostring(value), ID_CATCHER_PATTERN)
        local last_five = string.sub(table_id, #table_id - 4, # table_id)
        local first_part = string.sub(table_id, 1, #table_id - 5)
        return BOLD_WHITE .. "table" .. RESET .. LEFT_ANGLED .. "0x" .. first_part .. BOLD_WHITE .. last_five .. RESET .. RIGHT_ANGLED
    elseif type(value) == "userdata" then
        local userdata_id = string.match(tostring(value), ID_CATCHER_PATTERN)
        local typeof_userdata = typeof(value)
        if typeof_userdata ~= "userdata" then -- userdatas with custom names
            if typeof_userdata ~= "null" and typeof_userdata ~= "undefined" then -- json null & undefined dont need address printed
                return BOLD_MAGENTA .. typeof_userdata .. RESET .. LEFT_ANGLED .. MAGENTA .. userdata_id .. RESET .. RIGHT_ANGLED
            else
                return colors.bold.magenta(typeof_userdata)
            end
        else
            return BOLD_MAGENTA .. "userdata" .. RESET .. LEFT_ANGLED .. "0x" .. userdata_id .. RIGHT_ANGLED
        end
    else
        return tostring(value)
    end
end

local function stringify_table_path(current_table_path: { any }): string
    local table_path = BOLD_YELLOW .. "self" .. RESET
    for _, level in current_table_path do
        if typeof(level) == "string" then
            if string.match(level, IDENTIFIER_PATTERN) then
                table_path ..= "." .. level
            else
                table_path ..= LEFT_BRACKET .. GREEN .. '"' .. level .. '"' .. RESET .. RIGHT_BRACKET
            end
        elseif typeof(level) == "number" then
            table_path ..= LEFT_BRACKET .. BLUE .. level .. RESET .. RIGHT_BRACKET
        else
            table_path ..= LEFT_BRACKET .. stringify_simple_type(level) .. RIGHT_BRACKET
        end
    end
    return table_path
end

type TablePair = {
    key: any,
    val: any,
}

local function sort_table_pairs(unsorted_pairs: { TablePair }): { TablePair }
    local sorted_pairs: { TablePair } = table.create(#unsorted_pairs) :: { TablePair }
    local seen_indices: { [number]: boolean } = {} -- indices seen/visited in unsorted_pairs

    local numerical_pairs: { TablePair } = {} do
        for index, pair in unsorted_pairs do
            if type(pair.key) == "number" then
                seen_indices[index] = true
                table.insert(numerical_pairs, pair)
            end
        end
        table.sort(numerical_pairs, function(a: TablePair, b: TablePair)
            local isint_a = mlua.isint(a.key)
            local isint_b = mlua.isint(b.key)
            if isint_a and isint_b then
                return a.key < b.key
            elseif isint_a and not isint_b then
                return true
            elseif isint_b and not isint_a then
                return false
            else
                return a.key < b.key
            end
        end)
    end

    for _, v in numerical_pairs do
        table.insert(sorted_pairs, v)
    end

    local string_pairs: { TablePair } = {} do
        for index, pair in unsorted_pairs do
            if type(pair.key) == "string" then
                seen_indices[index] = true
                table.insert(string_pairs, pair)
            end
        end
        table.sort(string_pairs, function(a: TablePair, b: TablePair)
            local a_byte, b_byte = string.byte(a.key), string.byte(b.key)
            return a_byte < b_byte
        end)
    end

    for _, v in string_pairs do
        table.insert(sorted_pairs, v)
    end

    for index, pair in unsorted_pairs do
        if not seen_indices[index] then
            table.insert(sorted_pairs, pair)
        end
    end

    return sorted_pairs
end

local function process_pretty_values(
    value: any,
    -- seen_tables' keys are tables since they get uniquely hashed, value is stringified path
    seen_tables: { [{ [any]: any }]: string? }?,
    depth: number?,
    current_table_path: { any }? -- used to put table paths in seen_tables
): string
    seen_tables = seen_tables or {}
    local depth = depth or 0
    local current_table_path: { any } = current_table_path or {}
    local current_indent = string.rep("    ", depth)
    if type(value) == "table" then
        local value = value :: { [any]: any }
        local result = "{\n"

        if depth == 0 then
            seen_tables[value] = BOLD_YELLOW .. "self" .. RESET -- track top level cyclic tables
        end

        -- pretty printed tables should be printed in numerical, then alphabetical order (if possible)
        local unsorted_pairs: { TablePair } = {}
        local value_meta = getmetatable(value :: any) :: { [any]: any }?
        if value_meta then
            for k, v in pairs(value) do -- use pairs to avoid unexpectedly calling __call on tables w/ metatable
                table.insert(unsorted_pairs, { key = k, val = v }) 
            end
        else
            for k, v in value do
                table.insert(unsorted_pairs, { key = k, val = v }) 
            end
        end

        local sorted_pairs: { TablePair } = sort_table_pairs(unsorted_pairs)
        if #sorted_pairs == 0 then
            return "{}" -- special case empty tables
        end

        for _, pair in sorted_pairs do
            local key, val = pair.key, pair.val
            result ..= current_indent .. "    "

            -- stringify keys
            local key_type = type(key)
            if key_type == "number" then
                -- left pad array keys by prepending spaces in front of [
                -- so the = signs are aligned
                if #value > 10 and #value < 100 then
                    if key < 10 then
                        result ..= " "
                    end
                elseif #value > 100 and #value < 1000 then
                    if key < 10 then
                        result ..= "  "
                    elseif key < 100 then
                        result ..= " "
                    end
                elseif #value > 1000 then
                    if key < 10 then
                        result ..= "   "
                    elseif key < 100 then
                        result ..= "  "
                    elseif key < 1000 then
                        result ..= " "
                    end
                end
                result ..= LEFT_BRACKET
                       .. if mlua.isint(key) then BLUE .. tostring(key) .. RESET
                          else BOLD_BLUE .. tostring(key) .. RESET
                result ..= RIGHT_BRACKET
            elseif key_type == "string" then
                if string.match(key, IDENTIFIER_PATTERN) then
                    result ..= key
                else
                    result ..= LEFT_BRACKET .. GREEN .. '"' .. key .. '"' .. RESET .. RIGHT_BRACKET
                end
            elseif key_type == "table" then
                local seen_name: string? = seen_tables[key]
                if seen_name then
                    result ..= LEFT_BRACKET .. seen_name .. RIGHT_BRACKET
                elseif depth + 1 > 6 then
                    local tableid = string.match(tostring(key), ID_CATCHER_PATTERN)
                    result ..= LEFT_BRACKET .. colors.cyan(`table<0x{tableid}>`) .. RIGHT_BRACKET
                else
                    result ..= LEFT_BRACKET .. process_pretty_values(key, seen_tables, depth + 1, current_table_path) .. RIGHT_BRACKET
                end
            else
                result ..= LEFT_BRACKET .. process_pretty_values(key, seen_tables, depth + 1, current_table_path) .. RIGHT_BRACKET
            end

            result ..= " = "

            -- stringify values
            if type(val) == "table" then
                local val = val :: { [any]: any }
                local seen_name: string? = seen_tables[val]
                if seen_name then
                    result ..= seen_name
                elseif depth + 1 > 6 then
                    if next(val) == nil then -- table is empty
                        result ..= "{}"
                    else
                        result ..= "{" .. colors.codes.BOLD_WHITE .. "..." .. RESET .. "}"
                    end
                else
                    -- push key to current_table_path before recursion
                    table.insert(current_table_path, key)
                    seen_tables[val] = stringify_table_path(current_table_path) -- tostring(key)
                    result ..= process_pretty_values(val, seen_tables, depth + 1, current_table_path)
                    -- pop key from current_table_path after recursion
                    table.remove(current_table_path, #current_table_path)
                end
            elseif type(val) == "string" then -- don't recurse in extremely common case
                if string.find(val, "\n") then
                    if #val < 42 then
                        result ..= GREEN .. '"' .. string.gsub(val, "\n", "\\n") .. '"' .. RESET
                    else
                        result ..= "(newlined string starts on next line) " ..  GREEN .. "\n" .. val .. RESET .. "\n(end newlined string)"
                    end
                else
                    result ..= GREEN .. '"' .. val .. '"' .. RESET
                end
            elseif type(val) == "number" or type(val) == "boolean" then
                result ..= stringify_simple_type(val)
            else
                result ..= process_pretty_values(val, seen_tables, depth + 1, current_table_path)
            end
            
            result ..= DIM .. "," .. RESET .. "\n"
        end
        if value_meta then
            result ..= current_indent .. "    " .. RED .. "@metatable" .. RESET .. " = " .. process_pretty_values(value_meta, seen_tables, depth + 1, current_table_path) .. "\n"
        end
        result ..= current_indent .. "}"
        return result
    elseif type(value) == "string" and depth == 0 then -- calling print("hello world") shouldn't colorize
        return value
    else
        return stringify_simple_type(value)
    end
end

return {
    simple_print = process_raw_values,
    pretty_print = process_pretty_values,
}