--!optimize 2
--!nolint LocalShadow

local fs = require("@std/fs")
local json = require("@std/json")

type ResolveResult = {
    ok: true,
    resolved_path: string,
} | {
    ok: false,
    err: string,
}

local function resolve_require_path(requested_path: string): ResolveResult
    local current_path = ""
    local requiring_file_directory: string = "" do
        local debug_name: string = (debug :: any).info(3, "s") --[[ 
            this should give us the 
            debug name (set by luau.load().set_name) for the chunk that called require(),
            in the format `[string "./src/somewhere.luau"]`
        ]]
        local requiring_file = string.sub(debug_name, 10, -3) -- grabs the part between `[string "` and `"]`
        local file_parent = fs.path.parent(requiring_file)
        if file_parent ~= nil then
            requiring_file_directory = file_parent
        else
            return {
                ok = false,
                err = `couldn't find parent directory of requiring file '{requiring_file}'`,
            }
        end
    end
    -- strip .luau extension if user passed in a non-compliant path
    if string.sub(requested_path, #requested_path - 4) == ".luau" then
        current_path = string.sub(requested_path, 1, #requested_path - 5)
    else
        current_path = requested_path
    end
    
    if string.sub(current_path, 1, 1) == "@" then
        -- require aliases can be like "@alias/file" or "@alias"
        local extracted_alias = string.match(current_path, "^@([%w_]+)/?")
        if not extracted_alias then
            return {
                ok = false,
                err = `couldn't extract require alias from requested path '{current_path}'; is it a valid alias (like "@alias/path" or "@alias")?`
            }
        end
        -- now we need to find the closest .luaurc to the requiring directory
        -- we're supposed to start in the requiring file's dir and then traverse ancestors
        -- til we find a .luaurc
        local luaurc_path = "" do
            local current_dir = requiring_file_directory
            while true do
                local possible_luaurc_path = fs.path.join(current_dir, ".luaurc")
                if fs.path.exists(possible_luaurc_path) then
                    luaurc_path = possible_luaurc_path
                    break
                else
                    local parent_dir = fs.path.parent(current_dir)
                    if parent_dir ~= nil then
                        current_dir = parent_dir
                    else
                        return {
                            ok = false,
                            err = `alias '{extracted_alias}' in require '{current_path}' could not be resolved as no '.luaurc' config file was found in {requiring_file_directory}'s parent directories (recursive)`
                        }
                    end
                end
            end
        end
        
        local luaurc do
            local success, result: any = pcall(function()
                return json.readfile(luaurc_path)
            end)
            if success then
                luaurc = result :: { aliases: { [string]: string }? }
            else
                return {
                    ok = false,
                    err = `unable to read .luaurc at {luaurc_path}, is it a valid json/jsonc file? err: {tostring(result)}`,
                }
            end
        end

        local found_aliases: { [string]: string }? = if luaurc then luaurc.aliases else nil
        if found_aliases == nil then
            return {
                ok = false,
                err = `no aliases defined in .luaurc at '{luaurc_path}'`,
            }
        end
        
        local alias_found = false
        local checked_aliases: { string } = {}
        for alias, replacement_path in found_aliases do
            if alias == extracted_alias then
                alias_found = true
                -- strip trailing / in replacement alias if present
                if string.sub(replacement_path, #replacement_path - 1, #replacement_path - 1) == "/" then
                    replacement_path = string.sub(replacement_path, 1, #replacement_path - 2)
                end
                -- this should correctly handle cases like "@alias/file" and "@alias" without resulting in a double slash
                current_path = string.gsub(current_path, "@" .. alias, replacement_path)
            else
                table.insert(checked_aliases, alias)
            end
        end
        
        if not alias_found then
            return {
                ok = false,
                err = `alias '{extracted_alias}' was not found in the .luaurc at '{luaurc_path}'\naliases found: {table.concat(checked_aliases, ", ")}`
            }
        end
    elseif string.sub(current_path, 1, 1) == "." then
        -- relative paths like "./" or "../"
        local skip_prefix_length = if string.sub(current_path, 1, 2) == "./" then 2 else 3
        if skip_prefix_length == 3 then -- ../
            local parent_parent_dir: string? = fs.path.parent(requiring_file_directory)
            if parent_parent_dir then
                requiring_file_directory = parent_parent_dir
            else
                return {
                    ok = false,
                    err = `Couldn't find directory ../ relative to requiring file in require path '{requested_path}'`
                }
            end
        end
        local partial_path = string.sub(current_path, skip_prefix_length + 1, #current_path)
        current_path = fs.path.join(requiring_file_directory, partial_path)
    else -- invalid require path
        local err_message = 
            `Invalid require path format: Luau requires must start with a require alias (like "@alias/path" or "@alias") or relative path (ex. "./path" or "../path").\n` ..
            "Notes:\n"..
            "  - ending a require path with .luau is no longer recommended (but currently supported by seal)\n" ..
            `  - implicitly relative paths (ex. require("file.luau") without ./ or ../) are no longer allowed; see https://github.com/luau-lang/rfcs/pull/56"`
        return {
            ok = false,
            err = err_message,
        }
    end
    -- now that we've resolved a current_path, let's check if it's a dir w/ init.luau
    local resolved_dir = fs.find(current_path):try_dir()
    if resolved_dir then
        if resolved_dir:find("init.luau").type == "File" then
            current_path = fs.path.join(current_path, "init")
        else
            return {
                ok = false,
                err = `directory at '{current_path}' missing an init.luau; it cannot be required`,
            }
        end
    end
    current_path ..= '.luau'
    local resolved_file = fs.find(current_path)
    if resolved_file.type == "File" then
        return {
            ok = true,
            resolved_path = current_path,
        }
    else
        return {
            ok = false,
            err = `file not found: '{current_path}'`,
        }
    end
end

return resolve_require_path