--!nolint ImportUnused
--!nolint LocalUnused
--!nolint LocalShadow

-- hi!! welcome to sealey runtime ᶘ ᵔᴥᵔᶅ, here's some basics to get you set up 

-- Table of Contents (in vscode, ctrl c on a type to teleport there)
type TableOfContents = {
	setup: SealSetupAndRun,
	fs: Filesystem,
	env: EnvAndCliArgs,
	http: HttpRequests,
	process: SpawningChildProcesses,
}

-- standard libs are in @std/libname and can be required as so:
local fs = require("@std/fs")
local io = require("@std/io")
local env = require("@std/env")
local json = require("@std/json")
-- some libs are nested as so:
local input = require("@std/io/input")
local datetime = require("@std/time/datetime")

type SealSetupAndRun = {}; local back: TableOfContents do
	--[[
		To run a file with seal, run `seal <filenamewithext>` in a terminal.
		Command line arguments can be passed after the filenamewithext and can be accessed
		thru the "@std/env" library.

		To run arbitrary code with seal, run `seal '<src>'` in a terminal.

		To set up a new seal project in your current directory (you probably already did this), use
		`seal setup`

		To run the current project at its entry point, use `seal run`
	]]
end

--- To read and write files, use the @std/fs library!
type Filesystem = {}; local back: TableOfContents do
	-- read and write files
	local content = fs.readfile("./src/main.luau")
	fs.writefile("./src/myotherfile.txt", "hi")

	-- let's remove that file so we don't forget
	fs.removefile("./src/myotherfile.txt")

	-- use the path library to combine paths in an ergonomic, cross platform safe manner!
	local main_luau_path = fs.path.join(fs.path.cwd(), "src", "main.luau")

	-- use fs.find to grab a FileEntry or DirectoryEntry 
	-- (these represent entries on the filesystem and have nice methods!)
	local main_luau = fs.find(main_luau_path):try_file()
	if main_luau then
		local main_contents = main_luau:read()
	end

	-- iterate thru the files/dirs in a directory using fs.entries!
	for entry_path, entry in fs.entries("./src") do
		if entry.type == "File" then
			print(entry:read())
		elseif entry.type == "Directory" then
			print(entry:list())
		end
	end

	-- let's add some files and dirs to a new directory tree!
	local new_dir_path = fs.path.join(fs.path.cwd(), "newdir")

	-- step 1. ensure a directory exists before using it!
	local new_dir = fs.dir.ensure(new_dir_path)

	-- now we have a DirectoryEntry, so let's add some files and trees to it
	new_dir:add_file("config.json", json.encode_raw({ cats = 2 }))
	new_dir:add_tree("dependencies", fs.tree()
		:with_file("dep1.luau", "return 'hi'")
		:with_file("dep2.luau", 'return "bye"')
	)

	-- let's remove our new tree so we don't forget
	new_dir:remove()
end

--- to get, post, and send other http requests, use @std/net/http!
type HttpRequests = {}; local back: TableOfContents do
	local http = require("@std/net/http")
	type RandomDog = {
		message: string,
		success: boolean,
	}
	local dog_result: RandomDog = http.get({ 
		url = "https://dog.ceo/api/breeds/image/random"
	}):unwrap_json({
		message = "dog not found",
		success = "false",
	})
	
	local output = require("@std/io/output")
	
	print(`Your dog of the day is: {output.format(dog_result)}`)
end

--- to read command line arguments and access the current process's environment, use @std/env!
type EnvAndCliArgs = {}; local back: TableOfContents do
	local cli_args = env.args
	if cli_args[1] == "--help" then
		print("seal says help")
	end

	-- get and set environment variables
	local PATH: string? = env.getvar("PATH")
	env.setvar("hi", "true")
end

-- use @std/process to spawn child processes, shell commands, including in a nonblocking manner!
type SpawningChildProcesses = {}; local back: TableOfContents do
	local process = require("@std/process")

	-- to run a shell command, use process.shell; this uses some magic to figure out which shell you're currently using
	-- and spawns a shell command child process with your preferred shell
	local lslist = process.shell("ls"):unwrap("can't ls")

	-- to run any arbitrary process that blocks (the current VM) until it completes, use process.run
	local result = process.run {
		program = "git",
		args = { "--version" }
	}

	print(if result.ok then result.stdout
          else result.stderr)
	-- if you want to assume your process run attempt succeeded (and error otherwise), use :unwrap()
	local stdout = result:unwrap()

	-- to spawn a non-blocking child process and monitor its stdout, stderr, and even write to its stdin, use process.spawn
	local waiting_src = ([[
		local time = require("@std/time")
		time.wait(3)
		print("hi")
	]]):gsub("\n", "")

	local handle = process.spawn {
		program = `seal eval '{waiting_src}'`,
		shell = if env.os == "Windows" then "pwsh" else "sh"
	}

	local text = ""
	while handle:alive() do
		local new_text = handle.stdout:read(1)
		if new_text then
			text ..= new_text
		end
	end
end
