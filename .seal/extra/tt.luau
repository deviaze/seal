--[=[
	A `table` extension library written in pure Luau!

	Features common functions like `map`, `filter`, etc.
]=]
local tt = {}

--- check if you can iterate through table `t` normally (does it support __iter without blowing up?)
function tt.caniteratenormally(t: any): boolean
	local tmeta = getmetatable(t)
	-- stylua: ignore
	return if tmeta and tmeta.__call then false
		   elseif tmeta and tmeta.__iter then true
		   else true
end

--- length of an array-like or map-like table (# is incorrect on non array-like tables)
---
--- doesn't exactly work on mixed tables
function tt.len<K, V>(t: { [K]: V }): number
	if #t > 0 then
		return #t -- handles arrays and tables w/ custom length operator
	end

	local count = 0
	if tt.caniteratenormally(t) then
		for k, v in t do
			count += 1
		end
	else
		for k, v in pairs(t) do
			count += 1
		end
	end

	return count
end

--- Returns an array-like-table of `k`'s keys
function tt.keys<K>(t: { [K]: unknown }): { K }
	local keys = {}
	if tt.caniteratenormally(t) then
		for key, _ in t do
			-- LUAU FIXME: weird refinement of intersect<K, ~nil> not allowing table.insert overload
			-- this disappeared when returning keys and disappears when explicitly annotating keys: { K }
			table.insert(keys, key)
		end
	else
		for key, _ in pairs(t) do
			table.insert(keys, key)
		end
	end
	return keys
end

--- Returns an array-like-table of `k`'s keys
function tt.values<K>(t: { [K]: unknown }): { K }
	local values = {}
	if tt.caniteratenormally(t) then
		for _, value in t do
			-- LUAU FIXME: weird refinement of intersect<K, ~nil> not allowing table.insert overload
			table.insert(values, value)
		end
	else
		for _, value in pairs(t) do
			table.insert(values, value)
		end
	end
	return values
end

--[=[
	Create a new table by calling `mapper` repeatedly to transform the keys/values from `t` into a new table.

	You might need to explicitly specify the return type of your `mapper` function so luau gets your types correctly.
	
	## Usage
	```luau
	-- examples
	-- clean a little dataset
	local tt = require("@extra/tt")
	local data = read_csv(f)
	for i, row in data do
		local cleaned_row = tt.map(row, function(col, val)
			if k == "url" then
				return string.split(string.lower(k), "."), v
			elseif k == "ignore" then
				return nil, nil
			else
				return string.lower(k), v
			end
		end)
		data[i] = cleaned_row
	end

	-- map all init.luaus recursively by their file contents
	local fs = require("@std/fs")
	local str = require("@extra/str")
	local init_luaus = tt.map(fs.listdir("./", true), function(_, f: string): (string?, string?)
		if str.endswith(f, "init.luau") then
			return f, fs.readfile(f)
		else
			return nil, nil
		end
	end)
	```
]=]
function tt.map<K, V, NK, NV>(t: { [K]: V }, mapper: <K, V, NK, NV>(key: K, val: V) -> (NK?, NV?)): { [NK]: NV }
	local result: { [NK]: NV } = {}
	if tt.caniteratenormally(t) then
		for key, value in t do
			local new_key: NK?, new_value: NV? = (mapper :: any)(key, value) -- LUAU FIXME: blocked type return
			if new_key ~= nil and new_value ~= nil then
				-- LUAU FIXME: if new_key/new_value refinement break without ~= nil check
				result[new_key] = new_value
			end
		end
	else
		for key, value in pairs(t) do
			local new_key: NK?, new_value: NV? = (mapper :: any)(key, value) -- LUAU FIXME: blocked type return
			if new_key ~= nil and new_value ~= nil then
				-- LUAU FIXME: if new_key/new_value refinement break without ~= nil check
				result[new_key] = new_value
			end
		end
	end
	return result
end

function tt.arraymap<T>(t: { T }, mapper: (index: number, value: T) -> T?): { T }
	local result: { T } = {}

	if tt.caniteratenormally(t) then
		for index, value in t do
			local new_value = mapper(index, value)
			if new_value ~= nil then
				table.insert(result, new_value)
			end
		end
	else
		for index, value in ipairs(t) do
			local new_value = mapper(index, value)
			if new_value ~= nil then
				table.insert(result, new_value)
			end
		end
	end

	return result
end

function tt.filter<K, V>(t: { [K]: V }, filterer: (key: K, value: V) -> boolean): { [K]: V }
	local filtered = {}
	if tt.caniteratenormally(t) then
		for key, value in t do
			local should_include = filterer(key, value) :: any -- LUAU FIXME: blocked type return
			if should_include then
				filtered[key] = value
			end
		end
	else
		for key, value in pairs(t) do
			local should_include = filterer(key, value)
			if should_include then
				filtered[key] = value
			end
		end
	end
	return filtered
end

--- Returns a new table that contains only the selected keys (and values) of `t`
---
--- Use `tt.selectcolumns` instead if you want to return a strictly-typed struct using a type function
function tt.select<K, V>(t: { [K]: V }, columns: { K }): { [K]: V }
	local result = {}
	if tt.caniteratenormally(t) then
		for k, v in t do
			if table.find(columns, k) then
				result[k] = v
			end
		end
	else
		for k, v in pairs(t) do
			if table.find(columns, k) then
				result[k] = v
			end
		end
	end
	return result
end

--- type functions are cursed
type function Select(t: type, indexer: type?)
	local function getvaluetype(value: any)
		return
			-- stylua: ignore
			if value == "string"
			then types.string
			elseif value == "number" then types.number
			elseif value == "boolean" then types.boolean
			elseif value == "true" then types.singleton(true)
			elseif value == "false" then types.singleton(false)
			elseif (typeof)(value) == "type" then value :: any
			else types.unknown
	end
	if t:is("table") then
		local column_type = types.newtable()
		for k_type, v_type in t:properties() do
			local key: string? = nil
			if k_type:is("singleton") then
				key = k_type:value() :: string
			end
			local value = nil
			if v_type.read and v_type.read:is("singleton") then
				value = v_type.read:value() :: string
			elseif v_type.read and v_type.read:is("table") then
				value = v_type.read
			else
				print("value isn't a singleton or table, you probably need to :: cast it lol")
			end
			if key ~= nil and value ~= nil then
				column_type:setproperty(types.singleton(key), getvaluetype(value))
			end
		end
		if indexer then
			if indexer:is("table") then
				for k_type, v_type in indexer:properties() do
					local key: string? = nil
					if k_type:is("singleton") then
						key = k_type:value() :: string
					end
					local value = nil
					if v_type.read and v_type.read:is("singleton") then
						value = v_type.read:value() :: string
					elseif v_type.read and v_type.read:is("table") then
						value = v_type.read
					else
						print("value isn't a singleton or table, you probably need to :: cast it lol")
					end
					if key ~= nil and value ~= nil then
						column_type:setindexer(types.singleton(key), getvaluetype(value))
					end
				end
			end
		else
			column_type:setindexer(types.string, types.string)
		end
		return column_type
	end
	return types.unknown
end

--[=[
	Like `tt.select`, but you can customize the return type based on your data.

	```luau
	local data = {
		cats = 1,
		name = "Meow",
		age = 12,
		random = 2,
	}

	local cols = tt.selectcolumns(data,  {
		name = "string" :: "string", -- colums you want to select with their return type
		age = "number" :: "number",
	}, {
		string = "string" :: string, -- optional indexer
	})
	```
]=]
function tt.selectcolumns<K, V, S, I>(t: { [K]: V }, columns: S, indexer: I?): Select<S, I>
	local column_list = {}
	for k, v in columns :: { [K]: V } do
		table.insert(column_list, k :: any)
	end
	local result = tt.select(t, column_list)
	return (result :: any) :: Select<S, I>
end

--- Calls `mapper` to render `t` into a string, separated by `sep` (defaults to \n)
function tt.maptostring<K, V>(t: { [K]: V }, mapper: (key: K, value: V) -> string?, sep: string?): string
	if sep == nil then
		sep = "\n"
	end
	local result = ""
	if tt.caniteratenormally(t) then
		for k, v in t do
			local s = mapper(k, v) :: string?
			if s and sep then
				result ..= s .. sep
			end
		end
	else
		for k, v in pairs(t) do
			local s = mapper(k, v) :: string?
			if s and sep then
				result ..= s .. sep
			end
		end
	end
	return result
end

return tt
