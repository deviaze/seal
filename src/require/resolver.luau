--!optimize 2
--!nolint LocalShadow

--[[
	As of 2025-04-29 I still feel the custom error handling, ease of use, etc. of a custom
	require resolver makes more sense than using the Luau Require library at this time
	with its mlua implementation.
]]

type RequireResolveResult = {
	path: string,
} | {
	err: string,
}

local fs = require("@std/fs")
local json = require("@std/json")

type LuaurcConfig = {
	languageMode: "strict" | "nonstrict"?,
	lint: {
		[string]: boolean,
	}?,
	aliases: {
		[string]: string,
	}?,
}

type LuaurcAliases = {
	path: string,
	aliases: {
		[string]: string
	}
}

local function maptostring<K, V>(t: { [K]: V }, f: (key: K, val: V) -> string?): string
	local result = ""
	for k, v in t do
		local s = f(k, v)
		if s then
			result ..= "\n" .. s
		end
	end
	return result
end

local function str_startswith(s: string, component: string)
    return string.sub(s, 1, #component) == component
end
local function str_endswith(s: string, component: string)
    return string.sub(s, -#component) == component
end


local function get_aliases_by_luaurc(requiring_file_path: string): ({LuaurcAliases}?, string?)
	local aliases_by_luaurc: { LuaurcAliases } = {}

	local current_path = fs.path.parent(requiring_file_path)
	while current_path ~= nil do
		local possible_luaurc_path = fs.path.join(current_path :: string, ".luaurc")
		local contents, result = fs.file.try_read(possible_luaurc_path)
		if contents == nil then
			if result == "PermissionDenied" then
				return nil, `unable to read .luaurc at '{possible_luaurc_path}' because permission denied`
			end
		else
			local success, config = pcall(function(): LuaurcConfig
				return json.decode(contents)
			end)
			if not success then
				local err = config :: error
				return nil, `unable to decode .luaurc at '{possible_luaurc_path} due to err: {tostring(err)}'`
			end

			local config = config :: LuaurcConfig
			local current_luaurc: LuaurcAliases = {
				path = possible_luaurc_path,
				aliases = {},
			}
			if config.aliases then
				for key, replacement in config.aliases do
					local absolutized_replacement = fs.path.join(
						current_path :: string,
						if str_startswith(replacement, "./") then
							string.sub(replacement, 3)
						else 
							replacement
					)
					current_luaurc.aliases[key] = absolutized_replacement
				end
			end
			-- we want to 'push_front' our array so the closest aliases to the requiring_file
			-- get assigned last (so overwriting those in farther luaurcs)
			table.insert(aliases_by_luaurc, 1, current_luaurc)
		end
		current_path = fs.path.parent(current_path :: string)
	end
	
	return aliases_by_luaurc, nil
end

-- returns the expanded current_path with a replacement found from a .luaurc if one exists
-- if there's an error, the second return will be an error message
local function expand_aliases(requested_path: string, aliases_by_luaurc: { LuaurcAliases }): (string?, string?)
	local extracted_alias = string.match(requested_path, "^@([%w_]+)/?")
	if not extracted_alias then
		return nil, `couldn't extract require alias from requested path '{requested_path}'; is it a valid alias (like "@alias/path" or "@alias")?`
	end

	local aliases_found, luaurcs_found = 0, 0
	local luaurc_path: string? = nil
	local alias_replacement: string? = nil
	for priority, luaurc in aliases_by_luaurc do
		luaurcs_found += 1
		for key, replacement in luaurc.aliases do
			aliases_found += 1
			if key == extracted_alias then
				luaurc_path = luaurc.path
				alias_replacement = replacement
			end
		end
	end
	
	if alias_replacement then
		-- special case @file aliases
		if str_endswith(alias_replacement, ".luau") then
			return alias_replacement, nil
		end

		local rest_of_path = requested_path
		-- strip @alias prefix
		rest_of_path = string.sub(rest_of_path, #extracted_alias + 2, #rest_of_path)
		if str_startswith(rest_of_path, "/") then
			rest_of_path = string.sub(rest_of_path, 2, #rest_of_path)
		end
		if not str_endswith(alias_replacement, "/") then
			alias_replacement ..= "/"
		end
		return alias_replacement .. rest_of_path, nil
	else
		local function list_aliases(aliases: { [string]: string }, indent: string)
			return maptostring(aliases, function(key, replacement)
				return `{indent}'@{key}' => '{replacement}'`
			end)
		end

		local err_message =
			if luaurcs_found == 0 then
				`unable to expand alias '{extracted_alias}', no .luaurc configuration files found in requiring file's parent/ancestor directories`
			elseif luaurcs_found == 1 and aliases_found == 0 then
				`no aliases defined in the .luaurc at '{aliases_by_luaurc[1].path}' (looking for '{extracted_alias}')`
			elseif luaurcs_found == 1 and aliases_found > 0 then
				`alias '{extracted_alias}' not found in the .luaurc at '{aliases_by_luaurc[1].path}'; found aliases: {list_aliases(aliases_by_luaurc[1].aliases, "  ")}`
			else
				`alias '{extracted_alias}' not found in the following .luaurcs:` ..
					maptostring(aliases_by_luaurc, function(index: number, luaurc: LuaurcAliases)
						return `  {#aliases_by_luaurc - index}. '{luaurc.path}': {list_aliases(luaurc.aliases, "    ")}`
					end)
		return nil, err_message
	end
end

local function resolve(requested_path: string, test_debug_name: string?): RequireResolveResult
	local debug_name: string = test_debug_name or debug.info(3, "s") --[[ 
		this should give us the 
		debug name (set by luau.load().set_name) for the chunk that called require(),
		in the format `[string "./src/somewhere.luau"]`
	]]
	local requiring_file_path = string.sub(debug_name, 10, -3) -- grabs the part between `[string "` and `"]`

	local current_path = requiring_file_path
	
	local trim_after_index = 0
	if string.sub(requested_path, 1, 1) == "@" then
		local aliases_by_luaurc, err = get_aliases_by_luaurc(requiring_file_path)
		if err or aliases_by_luaurc == nil then
			return {
				err = err :: string
			}
		end
		local expanded_path, err = expand_aliases(requested_path, aliases_by_luaurc)
		if err or expanded_path == nil then
			return {
				err = err :: string,
			}
		end
		current_path = expanded_path
	elseif string.sub(requested_path, 1, 2) == "./" then
		current_path = fs.path.parent(current_path)
		if not current_path then
			return {
				err = `first level parent ./ doesn't exist relative to '{requiring_file_path}'`
			}
		end
		trim_after_index = 3
	elseif string.sub(requested_path, 1, 3) == "../" then
		current_path = fs.path.parent(current_path, 2)
		if not current_path then
			return {
				err = `second level parent ../ doesn't exist relative to '{requiring_file_path}'`
			}
		end
		trim_after_index = 4
	else
		return {
			err = `requested path '{requested_path}' does not start with './', '../', or '@', it cannot be required`
		}
	end

	-- trim ./ and ../ from requested_path
	if trim_after_index > 0 then
		current_path = fs.path.join(current_path, string.sub(requested_path, trim_after_index))
	end

	assert(current_path ~= nil, "current_path should not be nil at this point")

	local trying_to_require_directory = false
	local find_result: fs.FindResult = fs.find(current_path, { error_if_permission_denied = false })
	if find_result.type == "Directory" then
		current_path = fs.path.join(current_path, "init.luau")
		trying_to_require_directory = true
	elseif find_result.type == "NotFound" then
		current_path ..= ".luau"
	elseif find_result.type == "PermissionDenied" then
		return {
			err = `requested path '{requested_path}' resolved to '{current_path}', which cannot be required because permission denied`
		}
	end

	assert(current_path ~= nil, "current_path should not be nil at this point 2")

	if fs.path.exists(current_path) then
		return {
			path = current_path
		}
	elseif trying_to_require_directory then
		return {
			err = `directory at '{fs.path.parent(current_path)}' missing an 'init.luau', it cannot be required`
		}
	else
		return {
			err = `module not found: requested path '{requested_path}' resolved to '{current_path}', which wasn't found on the filesystem`
		}
	end
end

return resolve