local fs = require("@std/fs")
local io = require("@std/io")
local str = require("@std/str")
local env = require("@std/env")
local json = require("@std/json")
local colors = require("@std/colors")
local semver = require("@std/semver")
local process = require("@std/process")

local seal_setup = (require :: any)("@internal/setup") :: {
    extract_typedefs: (path: string) -> (),
    extract_extra: (path: string) -> (),
    GUIDED_TOUR_SRC: string,
    DEFAULT_CONFIG_SRC: string,
}

export type SetupConfig = {
    editor: "vscode" | "zed" | "none",
    codebase: "project" | "script",
    typedefs: "link" | "gen",
    git: boolean?,
    tour: boolean?,
}

local defaults = {
    default = {
        editor = "vscode",
        codebase = "project",
        typedefs = "gen",
        git = true,
        tour = true,
    } :: SetupConfig,
    project = {
        editor = "vscode",
        codebase = "project",
        typedefs = "gen",
        git = true,
        tour = false,
    } :: SetupConfig,
    ["script"] = {
        editor = "vscode",
        codebase = "script", -- puts all code in .seal/ instead of ./src/main.luau
        typedefs = "link", -- don't crowd .seal dir
        git = false, -- seal is being used as part of another repository, don't make a new repo
        tour = false,
    } :: SetupConfig
}

--- display error and exit
local function err(message: string): never
    print(`{colors.bold.red("seal setup:")} {colors.red(message)}`)
    return process.exit(1)
end

--- reads jsonc and confirms comment removal if jsonc contains comments
local read_jsonc

local update_luaurc
local update_vscode_settings
local update_home_seal_dir
local custom_setup_from_args

local extract

local function setup(config: SetupConfig?)
    if not config then
        config = custom_setup_from_args()
    end
    assert(config ~= nil, "config can't be nil here")

    local cwd = fs.dir.from(fs.path.cwd())
    local home_dir = fs.dir.from(fs.path.home())
    
    local typedefs_prefix = if config.typedefs == "link" then "~/" else "./"
    local globals_path = fs.path.join(typedefs_prefix, ".seal", "typedefs", "globals.d.luau")
    
    local luaurc_defaults = {
        languageMode = "strict",
        aliases = {
            std = fs.path.join(typedefs_prefix, ".seal", "typedefs", "std") .. "/",
            interop = fs.path.join(typedefs_prefix, ".seal", "typedefs", "interop") .. "/",
            extra = fs.path.join(typedefs_prefix, ".seal", "extra") .. "/",
        },
    }

    -- create or update .luaurc
    local luaurc_path = cwd:join(".luaurc")
    if fs.path.exists(luaurc_path) then
        local updated = update_luaurc(read_jsonc(luaurc_path), luaurc_defaults)
        json.writefile(luaurc_path, updated)
        print(`Updated .luaurc at '{luaurc_path}'`)
    else
        json.writefile(luaurc_path, luaurc_defaults)
    end

    -- editor specific settings
    if config.editor == "vscode" then
        local vscode_defaults = {
            ["luau-lsp.fflags.enableNewSolver"] = true,
            ["luau-lsp.fflags.enableByDefault"] = true,
            ["luau-lsp.platform.type"] = "standard",
            ["luau-lsp.types.definitionFiles"] = { globals_path },
            ["luau-lsp.inlayHints.parameterNames"] = "literals",
            ["workbench.editor.customLabels.patterns"] = {
                ["**/init.luau"] = "${dirname}",
                ["**/init.client.luau"] = "${dirname}",
                ["**/init.server.luau"] = "${dirname}"
            },
        }

        local settings_path = cwd:join(".vscode", "settings.json")
        if fs.path.exists(settings_path) and read_jsonc then
            local updated = update_vscode_settings(read_jsonc(settings_path), vscode_defaults)
            print(`Updated .vscode/settings.json at '{settings_path}'!`)
        else
            fs.dir.ensure(".vscode")
                :add_file("settings.json", json.encode(vscode_defaults))
        end
    elseif config.editor == "zed" then
        err("zed support not yet implemented")
    end

    -- extract typedefs and extra
    if config.typedefs == "gen" then
        extract(fs.dir.ensure(cwd:join(".seal")))
    elseif config.typedefs == "link" then
        local seal_dir = fs.dir(home_dir:join(".seal"))
        if not seal_dir then
            seal_dir = fs.dir.ensure(home_dir:join(".seal"))
            print(`Created a new .seal in your home directory: {seal_dir.path}!`)
            extract(seal_dir)
        else
            -- check if the existing .seal is compatible or needs to be updated
            local should_replace = update_home_seal_dir(seal_dir)
            if should_replace then
                local extracted = extract(seal_dir)
                if extracted then
                    print(`Linking {config.codebase} to updated .seal at {seal_dir.path}`)
                end
            else
                print(`Linking {config.codebase} to existing .seal at {seal_dir.path}`)
            end
        end
    end

    -- project codebases expect to be their own repos and have a ./src/main.luau
    -- script codebases expect to be part of an existing project & are self-contained in their .seal dir
    if config.codebase == "project" then
        fs.writetree(cwd:join("src"), fs.tree()
            :with_file("main.luau", 'print("Hello world!")')
        )
        -- either way (gen or link) we need a .seal dir in cwd for config.luau
        fs.dir.ensure(cwd:join(".seal"))
            :add_file("config.luau", seal_setup.DEFAULT_CONFIG_SRC)
    else
        local script_starter_imports = {
            `local fs = require("@std/fs")`,
            `local process = require("@std/process")`,
            "",
        }
        fs.dir.ensure(cwd:join(".seal"))
            :add_file("script.luau", table.concat(script_starter_imports, "\n"))
            :add_file(
                "config.luau",
                'return {\n    entry_path = "./.seal/script.luau"\n}'
            )
    end

    if config.tour then
        fs.dir.ensure(cwd:join(".seal"))
            :add_file("guided_tour.luau", seal_setup.GUIDED_TOUR_SRC)
    end

end

function update_luaurc(existing: json.JsonData, default: json.JsonData): json.JsonData
    existing.languageMode = existing.languageMode or "strict"
    -- update aliases
    local existing_aliases: { [string]: string } = existing.aliases or {}
    for alias, path in default.aliases :: { [string]: string } do
        if not existing_aliases[alias] then
            existing_aliases[alias] = path
        end
    end
    existing.aliases = existing_aliases
    return existing
end

function update_vscode_settings(existing: json.JsonData, default: json.JsonData): json.JsonData
    local function update(key: string, force: boolean?)
        local should_update = force or not existing[key]
        if should_update then
            existing[key] = default[key]
        end
        return should_update
    end
    update("luau-lsp.fflags.enableNewSolver", true)
    update("luau-lsp.fflags.enableByDefault", true)
    update("luau-lsp.platform.type", true)
    update("luau-lsp.inlayHints.parameterNames")
    update("workbench.editor.customLabels.patterns")
    if not update("luau-lsp.types.definitionFiles") then
        for _, definition_file in default["luau-lsp.types.definitionFiles"] do
            if not table.find(existing["luau-lsp.types.definitionFiles"], definition_file) then
                table.insert(existing["luau-lsp.types.definitionFiles"], definition_file)
            end
        end
    end
    return existing
end

function extract(seal_folder: fs.DirectoryEntry): boolean
    local typedefs_path = seal_folder:join("typedefs")
    if not fs.path.exists(typedefs_path) then
        fs.makedir(typedefs_path)
        seal_setup.extract_typedefs(typedefs_path)
    else
        return false
    end
    local extra_path = seal_folder:join("extra")
    if not fs.path.exists(extra_path) then
        fs.makedir(extra_path)
        seal_setup.extract_extra(extra_path)
    else
        return false
    end
    return true
end

local has_compatible_semver
function update_home_seal_dir(seal_dir: fs.DirectoryEntry): boolean
    local should_replace = true
    local config_luau = seal_dir:find("config.luau"):try_file()
    if config_luau then
        if has_compatible_semver(config_luau) then
            should_replace = false
        else
            err(`home's .seal has incompatible seal_version to this seal, not replacing its typedefs and exiting.`)
        end
    else
        seal_dir:add_file("config.luau", seal_setup.DEFAULT_CONFIG_SRC)
    end
    return should_replace
end

function has_compatible_semver(file: fs.FileEntry): boolean
    local function slightly_safe_require(path: string): { seal_version: string? }
        local s, res = pcall(function()
            return (require)(path)
        end)
        if s == true and typeof(res) == "table" then
            return res
        else
            return err(`Unable to require config.luau at '{path}', is it a valid config.luau file?`)
        end
    end
    local ver = slightly_safe_require(file.path).seal_version
    if ver and typeof(ver) == "string" then
        local self_semver = semver.from(_SEAL_VERSION)
        local other_semver = semver.from(ver)
        if other_semver:satisfies(`^{self_semver}`) then
            return true
        else
            return false
        end
    end
    return true
end

function custom_setup_from_args(): SetupConfig
    local args = env.args

    local editor = "vscode"
    local codebase = "project"
    local typedefs = "gen"
    local git = true
    local tour = false

    if #args == 0 then
        local function retry_prompt(prompt: string, valid_options: { string }): string
            local resp = str.trim(io.input.get(prompt .. ": ")):lower()
            -- allow numerical option corresponding to table index
            if tonumber(resp) then
                resp = valid_options[tonumber(resp) :: number]
            end
            if not table.find(valid_options, resp) then
                print(`{colors.red("Invalid option!")} Options: {table.concat(valid_options, " | ")}`)
                return retry_prompt(prompt, valid_options)
            else
                return resp
            end
        end

        print("Custom setup options:")

        local editor_prompt = `Choose an editor ({colors.blue("1: vscode")} | {colors.red("2: zed")}| 3: none)`
        editor = retry_prompt(editor_prompt, {"vscode", "code", "zed", "none"})

        local codebase_prompt = `Choose a codebase style ({colors.yellow("1: project")} | {colors.magenta("2: script")})`
        codebase = retry_prompt(codebase_prompt, {"project", "script"})

        local typedefs_prompt = `Generate typedefs or link to them? ({colors.green("1: gen")} | {colors.red("2: link")})`
        typedefs = retry_prompt(typedefs_prompt, {"gen", "link"})

        local git_prompt = `Make a git repository? (Y | n)`
        local git_response = retry_prompt(git_prompt, {"y", "n", ""})
        git = git_response == "" or git_response == "y"
    else
        for index, arg in args do
            if index % 2 == 0 then continue end
            local value = args[index + 1]
            if not value then
                err(`invalid number of args (got {#args})`)
            end
            if arg == "-e" or arg == "--editor" then
                editor =
                    if value == "vscode" or value == "code" or value == "1" then 
                        "vscode"
                    elseif value == "zed" or value == "2" then
                        "zed"
                    elseif value == "none" or value == "3" then
                        "none"
                    else
                        err(`invalid value for --editor (expected vscode/code/zed/none), got: {value}`)
            elseif arg == "-c" or arg == "--codebase" then
                codebase =
                    if value == "project" or value == "1" then
                        "project"
                    elseif value == "script" or value == "2" then
                        "script"
                    else
                        err(`invalid value for --codebase (expected project/script), got: {value}`)
            elseif arg == "-t" or arg == "--typedefs" then
                typedefs =
                    if value == "gen" or value == "generate" or value == "1" then
                        "gen"
                    elseif value == "link" or value == "2" then
                        "link"
                    else
                        err(`invalid value for --typedefs (expected gen/link), got: {value}`)
            elseif arg == "-g" or arg == "--git" then
                git = 
                    if value == "true" or value == "1" or value:lower() == "y" then
                        true
                    elseif value == "false" or value == "2" or value:lower() == "n" then
                        false
                    else
                        err(`invalid value for --git (expected true/false/Y/y/N/n), got: {value}`)
            else
                err(`invalid argument '{arg}'; expected any combination of --editor, --codebase, --typedefs, or --git`)
            end
        end
    end

    return {
        editor = editor,
        codebase = codebase,
        typedefs = typedefs,
        git = git,
        tour = tour,
    } :: SetupConfig
end

local check_confirm_comment_removal
function read_jsonc(path: string): json.JsonData
    local child = fs.path.child(path)
    local name = if child then `'{child}' at '{path}'` else `'{path}'`

    local content, result = fs.file.try_read(path)
    if result == "NotFound" then
        err(`{name} not found`)
    elseif result == "PermissionDenied" then
        err(`cannot access {name} (permission denied)`)
    elseif content and result == "Ok" then
        if not check_confirm_comment_removal(content, name) then
            process.exit(0)
        end
        local _, parsed = pcall(function()
            return json.decode(content)
        end)
        if typeof(parsed) == "error" then
            err(`cannot decode {name}; is it valid jsonc?`)
        else
            return parsed
        end
    end
    error("unreachable")
end

function check_confirm_comment_removal(content: string, name: string): boolean
    local function has_comments(content: string): boolean
        for _, line in str.splitlines(content) do
            if string.match(line, '//[%w%s%p[^"]]$') then
                return true
            end
        end
        return false
    end
    if has_comments(content) then
        local response = str.trim(io.input.get(`All comments in {name} will be stripped; continue? (Y/n)`)):lower()
        return response == "y" or response == ""
    else
        return true
    end
end

return {
    defaults = defaults,
    setup = setup,
}
