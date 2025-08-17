local fs = require("@std/fs")
local io = require("@std/io")
local str = require("@std/str")
local env = require("@std/env")
local json = require("@std/json")
local luau = require("@std/luau")
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

local get_updated_luaurc
local get_updated_vscode_settings
local update_seal_dir
local extract
local is_git_repo
local run_git_init
local custom_setup_from_args

local function setup(config: SetupConfig?)
    if not config then
        config = custom_setup_from_args()
    end
    assert(config ~= nil, "config can't be nil here")

    local cwd = fs.dir.from(fs.path.cwd())
    local home_dir = fs.dir.from(fs.path.home())
    
    local prefix = if config.typedefs == "link" then "~/" else "./"
    local globals_path = fs.path.join(prefix, ".seal", "typedefs", "globals.d.luau")
    
    local luaurc_defaults = {
        languageMode = "strict",
        aliases = {
            std = fs.path.join(prefix, ".seal", "typedefs", "std") .. "/",
            interop = fs.path.join(prefix, ".seal", "typedefs", "interop") .. "/",
            extra = fs.path.join(prefix, ".seal", "extra") .. "/",
        },
    }

    -- create or update .luaurc
    local luaurc_path = cwd:join(".luaurc")
    if fs.path.exists(luaurc_path) then
        local updated = get_updated_luaurc(read_jsonc(luaurc_path), luaurc_defaults)
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
            local updated = get_updated_vscode_settings(read_jsonc(settings_path), vscode_defaults)
            json.writefile(settings_path, updated)
            print(`Updated .vscode/settings.json at '{settings_path}'`)
        else
            fs.dir.ensure(cwd:join(".vscode"))
                :add_file("settings.json", json.encode(vscode_defaults))
        end
    elseif config.editor == "zed" then
        err("zed support not yet implemented")
    end

    -- extract typedefs and extra
    if config.typedefs == "gen" then
        local seal_dir_path = cwd:join(".seal")
        local seal_dir = fs.dir(seal_dir_path)
        if seal_dir then
            update_seal_dir(seal_dir)
        else
            extract(fs.dir.ensure(seal_dir_path))
        end
    elseif config.typedefs == "link" then
        local seal_dir_path = home_dir:join(".seal")
        local seal_dir = fs.dir(seal_dir_path)
        if seal_dir then
            update_seal_dir(seal_dir)
        else
            extract(fs.dir.ensure(seal_dir_path))
            print(`Created a new ~/.seal dir at: {seal_dir_path}!`)
        end
    end

    -- project codebases expect to be their own repos and have a ./src/main.luau
    -- script codebases expect to be part of an existing project & are self-contained in their .seal dir
    if config.codebase == "project" then
        local src_dir = fs.dir(cwd:join("src"))
        if not src_dir then
            fs.writetree(cwd:join("src"), fs.tree()
                :with_file("main.luau", 'print("Hello world!")')
            )
        end
        -- either way (gen or link) we still need a .seal dir at the cwd for config.luau
        local seal_dir = fs.dir.from(cwd:join(".seal"))
        if not fs.path.exists(seal_dir:join("config.luau")) then
            seal_dir:add_file(
                "config.luau", 
                (seal_setup.DEFAULT_CONFIG_SRC:gsub("<SEAL_VERSION_REPLACE>", _SEAL_VERSION)
            )
        )
        end
    else
        local script_starter_imports = {
            `local fs = require("@std/fs")`,
            `local process = require("@std/process")`,
            "",
        }
        if not fs.path.exists(cwd:join(".seal", "config.luau")) then
            fs.dir.ensure(cwd:join(".seal"))
                :add_file("script.luau", table.concat(script_starter_imports, "\n"))
                :add_file(
                    "config.luau",
                    'return {\n    entry_path = "./.seal/script.luau"\n}'
                )
        end
    end

    if config.git then
        if not is_git_repo(cwd) then
            run_git_init()
        end
    end

    if config.tour then
        fs.dir.ensure(cwd:join(".seal"))
            :add_file("guided_tour.luau", seal_setup.GUIDED_TOUR_SRC)
    end

end

function get_updated_luaurc(existing: json.JsonData, default: json.JsonData): json.JsonData
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

function get_updated_vscode_settings(existing: json.JsonData, default: json.JsonData): json.JsonData
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

local should_replace_typedefs
local should_replace_extra
function update_seal_dir(seal_dir: fs.DirectoryEntry)
    local config_luau = seal_dir:find("config.luau"):try_file()
    if not config_luau then
        seal_dir:add_file(
            "config.luau", 
            (seal_setup.DEFAULT_CONFIG_SRC:gsub("<SEAL_VERSION_REPLACE>", _SEAL_VERSION))
        )
    end
    config_luau = seal_dir:expect_file("config.luau")
    if should_replace_typedefs(config_luau) then
        local typedefs_path = seal_dir:join("typedefs")
        fs.dir.try_remove(typedefs_path)
        fs.makedir(typedefs_path)
        seal_setup.extract_typedefs(typedefs_path)
    end
    if should_replace_extra(seal_dir) then
        local extra_path = seal_dir:join("extra")
        fs.dir.try_remove(extra_path)
        fs.makedir(extra_path)
        seal_setup.extract_extra(extra_path)
    end
end

function should_replace_typedefs(config_luau: fs.FileEntry)
    local function slightly_safe_eval(path: string): { seal_version: string? }
        local src = fs.readfile(path)
        local res = luau.eval(src, { stdlib = "seal" }) -- allow seal stdlib in case test functions or smth need it
        if typeof(res) == "error" then
            err(`Error reading your config.luau file at {path}: {res}`)
            error("unreachable")
        else
            return res :: { seal_version: string? }
        end
    end
    local function parse_semver(ver: string): semver.Semver
        local s, sem = pcall(function(): semver.Semver
            return semver.from(ver)
        end)
        if typeof(sem) == "error" then
            err(`Can't parse semver in config.luau at '{config_luau.path}' due to err: {tostring(sem)}`)
            error("unreachable")
        else
            return sem
        end
    end

    local should_replace = false
    local existing_version = slightly_safe_eval(config_luau.path).seal_version
    if existing_version then
        local existing_semver = parse_semver(existing_version)
        local seal_semver = semver.from(_SEAL_VERSION)

        if seal_semver < existing_semver then
            err(`Can't link to a newer version of seal! \n\z
                Your .seal dir was generated by seal {existing_semver}, but you're using seal {seal_semver}. \n\z
                If you want to make a project using this version of seal, generate a .seal locally using seal setup custom or delete your .seal dir and try again`)
        end

        if seal_semver:satisfies(`^{existing_semver}`) then
            if seal_semver > existing_semver then
                should_replace = true
            elseif seal_semver == existing_semver then
                should_replace = false
            end
        else
            local response = str.trim(
                io.input.get(`Update ~/.seal/typedefs from {existing_semver} -> {seal_semver}? \z
                This might break your existing linked typedefs if seal made breaking changes. (y/N)`)
            ):lower()
            should_replace = if response == "y" then true else false
        end
    else
        should_replace = true
    end
    return should_replace
end

function should_replace_extra(seal_dir: fs.DirectoryEntry): boolean
    local extra_dir = fs.dir(seal_dir:join("extra"))
    if not extra_dir then
        return true
    end
    local response = str.trim(io.input.get(
        `Replace '.seal/extra' dir at {extra_dir.path}? This may remove any customizations you've made to @extra libraries. \z(Y/n): `
    )):lower()
    return response == "y" or response == ""
end

function is_git_repo(cwd: fs.DirectoryEntry): boolean
    return cwd:find(".git"):try_dir() ~= nil
end

function run_git_init(): boolean
    local git_version_result = process.run { -- check if git exists on system
        program = "git",
        args = { "--version" },
    }
    if not git_version_result.ok then
        print("seal setup - version control software `git` not found on your system - I recommend installing it! (https://git-scm.com/downloads)")
        return false
    end

    local git_init_result = process.run {
        program = "git",
        args = { "init" },
    }
    if not git_init_result.ok then
        print("seal setup - error initializing git repository")
        return false
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
        local in_block_comment = false
        for _, line in str.splitlines(content) do
            -- remove string literals to avoid false positives
            local stripped = string.gsub(line, '".-"', '')
            if in_block_comment then
                if string.find(stripped, "%*/") then
                    in_block_comment = false
                end
                return true
            end
            -- match single-line comments
            if string.match(stripped, "^%s*//") or string.match(stripped, "[^:]//") then
                return true
            end
            -- match inline block comments
            if string.match(stripped, "/%*.*%*/") then
                return true
            end
            -- detect start of multiline block comment
            if string.match(stripped, "/%*") and not string.match(stripped, "%*/") then
                in_block_comment = true
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
