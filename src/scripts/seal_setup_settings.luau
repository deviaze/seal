--!nolint LocalShadow
--- configures a directory's .vscode and .luaurc settings for seal
-- doesn't handle spawning in .typedefs (that's handled in rust)
local fs = require("@std/fs")
local json = require("@std/json")
local process = require("@std/process")

type DirectoryEntry = fs.DirectoryEntry
type FileEntry = fs.FileEntry

local default_vscode_settings = {
	["luau-lsp.fflags.enableNewSolver"] = true,
	["luau-lsp.platform.type"] = "standard",
	["luau-lsp.types.definitionFiles"] = { "./.typedefs/globals.d.luau" },
}

local default_luaurc = {
	languageMode = "strict",
	aliases = {
		std = "./.typedefs/std/",
		interop = "./.typedefs/interop/",
	} :: { [string]: string }
}

local function update_vscode_settings()
	local vscode_dir = fs.dir.ensure(".vscode")
	local settings_json = vscode_dir:find("settings.json"):try_file()
	if settings_json then
		local vscode_settings = json.readfile(settings_json.path)
		for key, value in default_vscode_settings do -- replace nonexisting keys with their defaults
			if not vscode_settings[key] then
				vscode_settings[key] = if type(value) == "table" then table.clone(value) else value
			end
		end
		local current_definition_files: { string } = vscode_settings["luau-lsp.types.definitionFiles"]
		for _, definition_path in default_vscode_settings["luau-lsp.types.definitionFiles"] do
			if not table.find(current_definition_files, definition_path) then
				table.insert(current_definition_files, definition_path)
			end
		end
		vscode_dir:add_file("settings.json", json.encode(vscode_settings))
	else
		vscode_dir:add_file("settings.json", json.encode(default_vscode_settings))
	end
end

local function update_luaurc()
	if fs.path.exists(".luaurc") then
		local luaurc = json.readfile(".luaurc")
		if not luaurc.languageMode then
			luaurc.languageMode = default_luaurc.languageMode
		end
		luaurc.aliases = (luaurc.aliases or {}) :: { [string]: string }
		for key, value in default_luaurc.aliases do
			luaurc.aliases[key] = value -- overwriting @std/interop/extra is ok
		end
		json.writefile(".luaurc", luaurc)
	else
		json.writefile(".luaurc", default_luaurc)
	end
end

local function create_src_dir()
	if not fs.path.exists("src") then
		fs.writetree("src", fs.tree()
			:with_file("main.luau", 'print("Hello world!")')
		)
	end
end

local function init_git_repository()
	if not fs.path.exists(".git") then
		local git_version_result = process.run { -- check if git exists on system
			program = "git",
			args = { "--version" },
		}
		if git_version_result.ok then
			local git_init_result = process.run {
				program = "git",
				args = { "init" },
			}
			if not git_init_result.ok then
				print("seal setup - error initializing git repository")
			end
		else
			print("seal setup - version control software `git` not found on your system - I recommend installing it! (https://git-scm.com/downloads)")
		end
	end
end

update_vscode_settings()
update_luaurc()
create_src_dir()
init_git_repository()
