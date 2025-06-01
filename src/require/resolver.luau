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
local str = require("@std/str")
local env = require("@std/env")
local json = require("@std/json")
local colors = require("@std/colors")

type LuaurcConfig = {
	languageMode: "strict" | "nonstrict"?,
	lint: {
		[string]: boolean,
	}?,
	aliases: {
		[string]: string,
	}?,
}

export type LuaurcAliases = {
	path: string,
	aliases: {
		[string]: string
	}
}

local function maptostring<K, V>(t: { [K]: V }, f: (key: K, val: V) -> string?): string
	local result = ""
	for k, v in t :: any do -- LUAU FIXME: solver fails to complete
		local s = f(k, v) -- TypeError: Function only returns 0 values, but 1 is required here
		if s then
			result ..= "\n" .. s -- TypeError: Type function instance refine<*blocked-14022*, ~(false?)> is uninhabited
		end
	end
	return result
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
						if str.startswith(replacement, "./") then
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

	if str.endswith(requiring_file_path, "init.luau") then
		-- @self should refer to the direct parent dir of an init.luau
		-- regardless of whichever meaning of ./relative we use in seal 
		table.insert(aliases_by_luaurc, 1, {
			path = "self alias for init.luau (https://github.com/luau-lang/rfcs/pull/109)",
			aliases = {
				self = fs.path.parent(requiring_file_path) :: string
			}
		})
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
		if str.endswith(alias_replacement, ".luau") then
			return alias_replacement, nil
		end

		local rest_of_path = requested_path
		-- strip @alias prefix
		rest_of_path = string.sub(rest_of_path, #extracted_alias + 2, #rest_of_path)
		if str.startswith(rest_of_path, "/") then
			rest_of_path = string.sub(rest_of_path, 2, #rest_of_path)
		end
		if not str.endswith(alias_replacement, "/") then
			alias_replacement ..= "/"
		end
		return alias_replacement .. rest_of_path, nil
	else
		local function list_aliases(aliases: { [string]: string }, indent: string)
			return maptostring(aliases, function(key, replacement)
				return ` {indent}- '{key}' => {colors.codes.GREEN}"{replacement}"{colors.codes.RESET}`
			end)
		end

		local function display_aliases_by_luaurc(aliases_by_luaurc: { LuaurcAliases }): string
			local result = ""
			local current_luaurc = 1
			for i = #aliases_by_luaurc, 1, -1 do -- go backwards
				local luaurc: LuaurcAliases = aliases_by_luaurc[i]
				result ..= `\n  {current_luaurc}: {luaurc.path}: {list_aliases(luaurc.aliases, "    ")}`
				current_luaurc += 1
			end
			return result
		end

		local err_message = `unable to resolve requested path '{requested_path}'\n  ` .. colors.codes.RESET ..
			if luaurcs_found == 0 then
				`unable to expand alias '{extracted_alias}', no .luaurc configuration files found in requiring file's parent/ancestor directories`
			elseif luaurcs_found == 1 and aliases_found == 0 then
				`no aliases defined in the .luaurc at '{aliases_by_luaurc[1].path}' (looking for '{extracted_alias}')`
			elseif luaurcs_found == 1 and aliases_found > 0 then
				`alias '{colors.bold.red(extracted_alias)}' not found in the .luaurc at '{aliases_by_luaurc[1].path}'; found aliases: {list_aliases(aliases_by_luaurc[1].aliases, "  ")}`
			else
				`alias '{colors.bold.red(extracted_alias)}' not found in the following .luaurcs:` .. display_aliases_by_luaurc(aliases_by_luaurc)
		return nil, err_message
	end
end

--[=[
	Tries to resolve `requested_path` from the requiring file's debug name (optionally overridden by `override_debug_name`),
	using the new `init.luau` semantics if `try_new_init_luau_semantics` is enabled.

	seal uses the new semantics by default, and falls back to the old semantics if a module wasn't found.
	In either case, the alias @self should always be available and consistently refer to init.luau's direct parent directory
]=]
local function resolve(
	requested_path: string,
	override_debug_name: string?,
	try_new_init_luau_semantics: boolean? --[[
		seal defaults to new init.luau semantics since that's what the newest luau-lsp currently supports
		- old semantics = ./meow is in the same directory as the requiring init.luau
		- new semantics = ./meow is in the parent directory of the requiring init.luau
		but if the new semantics don't work (file not found) we try the old one by calling resolve again recursively
		note that seal supports @self in either case
	]]
): RequireResolveResult
local debug_name: string = override_debug_name or debug.info(3, "s") --[[ 
	this should give us the 
	debug name (set by luau.load().set_name) for the chunk that called require(),
	in the format `[string "./src/somewhere.luau"]`
	]]
	local requiring_file_path = string.sub(debug_name, 10, -3) -- grabs the part between `[string "` and `"]`

	local try_new_init_luau_semantics = -- luau lsp has updated, so let's use new semantics by default
		if try_new_init_luau_semantics == nil or try_new_init_luau_semantics == true then true else false
	local use_new_init_luau_semantics = -- we only want to use new init.luau semantics if
		try_new_init_luau_semantics and str.endswith(requiring_file_path, "init.luau")

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
		current_path = fs.path.parent(current_path, if use_new_init_luau_semantics then 2 else 1)
		trim_after_index = 3
	elseif string.sub(requested_path, 1, 3) == "../" then
		current_path = fs.path.parent(current_path, if use_new_init_luau_semantics then 3 else 2)
		trim_after_index = 4
	else
		return {
			err = `requested path '{requested_path}' does not start with './', '../', or '@'; it cannot be required.\n`
			    .. '  note: implicitly relative requires (like require("sibling")) are no longer supported by seal'
		}
	end

	if current_path == nil then
		return {
			err = `one or more parents could not be extracted from '{requested_path}'; is the requested path formatted correctly?`
		}
	end

	-- trim ./ and ../ from requested_path
	if trim_after_index > 0 then
		current_path = fs.path.join(current_path, string.sub(requested_path, trim_after_index))
	end

	if env.os == "Windows" then
		-- it's possible we could've accidentally mixed \ and /s when combining paths
		-- (especially when using alias replacements, which always use /)
		current_path = fs.path.normalize(current_path)
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
			err = `'{requested_path}' can't be required: requested path resolved to '{current_path}', \z
			which cannot be required because permission denied :(`
		}
	end

	assert(current_path ~= nil, "current_path should not be nil at this point 2")

	if fs.path.exists(current_path) then
		return {
			path = current_path
		}
	elseif trying_to_require_directory then
		return {
			err = `'{requested_path}' can't be required: requested path resolved to '{fs.path.parent(current_path)}', \z
			which is a directory missing its init.luau (needed to treat filesystem directories as 'modules' in the Luau Require System)`
		}
	elseif try_new_init_luau_semantics and str.endswith(requiring_file_path, "init.luau") then
		return resolve(requested_path, debug_name, false)
	elseif use_new_init_luau_semantics == false and string.match(requested_path, "%.%.?/") then
		return {
			err = `module '{requested_path}' not found: relative require from the init.luau at '{requiring_file_path}' resolved to '{current_path}', which wasn't found on the filesystem.\n`
			     .. "  note on require resolution semantics:\n"
			     .. "    - old: './meow' means meow.luau is in the same directory as the requiring init.luau\n"
				 .. "    - new: './meow' means meow.luau is in the parent directory of the requiring init.luau\n"
				 .. "    + for more information, check out: https://github.com/luau-lang/rfcs/pull/109\n"
				 .. "  seal tried both old and new require resolution semantics but was unable to find your module :("
		}
	else
		return {
			err = `module '{requested_path}' not found: requested path resolved to '{current_path}', which wasn't found on the filesystem`
		}
	end
end

return {
	resolve = resolve,
	get_aliases = get_aliases_by_luaurc,
	expand_aliases = expand_aliases,
}