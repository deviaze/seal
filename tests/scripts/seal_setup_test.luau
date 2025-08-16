-- we need to make sure running `seal setup` actually works
-- and that requires in a new project outside seal's repository continue
-- to work properly

local fs = require("@std/fs")
local env = require("@std/env")
local json = require("@std/json")
local process = require("@std/process")
local format = require("@std/io/format")

local project_path = 
	if env.os == "Android" then fs.path.join(fs.path.home(), "sealproject")
	else fs.path.join(fs.path.home(), "Documents", "sealproject")

if fs.path.exists(project_path) then
	fs.removetree(project_path) -- don't tell me abt toctou pls
end
local sealproject = fs.dir.ensure(project_path, true)

local absolute_executable_path = fs.path.canonicalize(env.executable_path)

local function sealsetup()
	local seal_setup_result = process.run {
		program = absolute_executable_path,
		args = { "setup" },
		cwd = project_path,
	}
	if not seal_setup_result.ok then
		print(seal_setup_result)
	end
	assert(seal_setup_result.ok == true, "seal setup not ok?")

	local function projectpathexists(...: string)
		return fs.path.exists(sealproject:join(...))
	end

	-- assert(fs.path.exists(sealproject:join("src")), "sealproject/src dir not exists?")
	assert(projectpathexists("src"), "sealproject/src dir not exists?")
	assert(projectpathexists("src", "main.luau"), "sealproject/src/main.luau not exists?")
	assert(projectpathexists(".seal"), "sealproject/.seal not exists?")
	assert(projectpathexists(".seal", "config.luau"), "sealproject/.seal/config.luau not exists?")
	assert(projectpathexists(".luaurc"), "sealproject/.luaurc not exists?")
	assert(projectpathexists(".vscode", "settings.json"), "sealproject/.vscode/settings.json not exists?")
end

sealsetup()

local function testsealsetuprequires()
	local srcs = {
		relative = [[
			return function()
				return "alabama"
			end
		]],
		relative_caller = [[
			local relative = require("./very_relative")
			assert(relative() == "alabama", "not alabama")
			print("yes")
		]],
		alias_caller = [[
			local rng = require("@custom/get_random_number")
			local n = rng()
			assert(type(n) == "number", "n not a number, what?")

			local wrapper = require("@custom/wrapper")
			assert(wrapper == "initted")
			print("initted")
		]],
		get_random_number = [[
			return function()
				return math.random(1, 200)
			end
		]],
		-- TODO: deal with require resolution change and @self
		wrapper_init = [[
			local other = require("./other")
			print(other.call("hi"))
			return "initted"
		]],
		wrapper_other = [[
			local other = {}
			function other.call(saying: string): string
				if saying == "hi" then
					return "bye"
				end
			end
			return other
		]]
	}

	-- add a relative require and callers to sealproject/src
	sealproject:expect_dir("src")
		:add_file("very_relative.luau", srcs.relative)
		:add_file("relative_caller.luau", srcs.relative_caller)
		:add_file("alias_caller.luau", srcs.alias_caller)

	local custom_modules = fs.tree()
		:with_file("get_random_number.luau", srcs.get_random_number)
		:with_tree("wrapper", fs.tree()
			:with_file("init.luau", srcs.wrapper_init)
			:with_file("other.luau", srcs.wrapper_other)
		)

	sealproject:add_tree("libraries", fs.tree()
		:with_tree("custom_modules", custom_modules)
	)

	-- add the useful alias to the project's .luaurc
	local luaurc_path = sealproject:join(".luaurc")
	local config = json.readfile(luaurc_path) :: { aliases: { [string]: string }}
	config.aliases["custom"] = "./libraries/custom_modules/"
	json.writefile(luaurc_path, config :: any)

	local function run_file(path: string)
		local result = process.run {
			program = absolute_executable_path,
			args = { path },
			cwd = project_path,
		}
		assert(result.ok, `running 'seal {path}' not okay?, got: \n{format(result)}`)
		return result:unwrap()
	end

	assert(run_file("./src/relative_caller.luau"):match("yes"), "relative should be yes")
	local alias_caller_output = run_file("./src/alias_caller.luau")
	assert(alias_caller_output:match("bye"), "bye should have been printed because it's in wrapper/init.luau")
	assert(alias_caller_output:match("initted"), "alias thing should have been initted")
end

testsealsetuprequires()

sealproject:remove()
