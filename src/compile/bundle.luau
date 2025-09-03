-- yes, this is in luau.
local fs = require("@std/fs")
local luau = require("@std/luau")


-- type BundleResponse = {
--     res: string
-- } | {
--     err: string
-- }

type BundleResponse = string | error

local err = {} :: {
    message: (m: string) -> error
}

local function find_entry_path(project_path: string): BundleResponse
    local project_dir = fs.dir(project_path)
    if not project_dir then
        return err.message(`path at '{project_path} doesn't exist (so it can't be compiled)`)
    end
    local seal_dir = project_dir:find(".seal"):try_dir()
    if not seal_dir then
        return {
            err = `directory at '{project_path}' isn't a seal project (doesn't contain a .seal dir), so it cannot be compiled`
        }
    end
    local config_luau = seal_dir:find("config.luau"):try_file()
    if not config_luau then
        return {
            err = `seal project at '{project_path}' missing its .seal/config.luau, it cannot be compiled because it doesn't have an entry_path`
        }
    end
    local config = luau.eval(config_luau:read())
    if typeof(config) == "error" then
        return {
            err = `config.luau at '{config_luau.path}' couldn't be evaluated due to err: {tostring(config)}`
        }
    elseif typeof(config) ~= "table" then
        return {
            err = `config.luau at '{config_luau.path}' returned unexpected datatype; expected table, got: {typeof(config)}`
        }
    else
        config = config :: {
            entry_path: string?
        }
    end

    local found_entry_path = config.entry_path
    
end

local function bundle(project_path: string): BundleResponse
    local entry_path: string do
        
    end

end
