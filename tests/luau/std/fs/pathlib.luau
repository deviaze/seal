local fs = require("@std/fs")
local env = require("@std/env")

local path = fs.path

if env.os ~= "Windows" then
	local function simplepaths()
		local np = path.join("src", "main.luau")
		if env.os ~= "Windows" then
			assert(np == "src/main.luau", "linux/unix simple paths broke")
		else
			assert(np == [[src\main.luau]], "windows paths broke")
		end
		-- print(path.canonicalize(np))
		if env.os ~= "Windows" then
			assert(path.absolutize(np):match("seal/src/main.luau"))
		end
	end
	
	simplepaths()
	
	local function absolutize()
		local cwd = path.child(path.absolutize("."))
		assert(cwd == "seal", "cwd is not seal for some reason")	
	end
	
	absolutize()
	
	local function canonicalize()
		-- path.canonicalize can't canonicalize nonexistent path
		local s, f = pcall(function()
			path.canonicalize("./existence.jpeg")
		end)
		assert(s == false, "canonicalize cannot succeed on ./existence.jpeg??")
		assert(tostring(f):match("doesn't exist on the filesystem"))
	end
	
	canonicalize()
	
	local function appendingpaths()
		local partial_path = path.join("std", "fs")
		local joined = path.join(partial_path)
		assert(joined == "std/fs", "simplest path join broke?")
	
		local slashes_are_bad = `/{path.join("project", "src", "ui")}/someplace.html`
		local slashed = path.join("home", slashes_are_bad)
		assert(slashed == "home/project/src/ui/someplace.html", `Leading slashes not stripped (if not first arg)?: '{slashed}'`)
	end
	
	appendingpaths()
	
	local function child()
		local p = path.join("hi", "how", "are/you")
		assert(path.child(p) == "you", "path.child 1 broke")
	end
	
	child()
end

local function getscriptpath()
	local script_path = script:path()
	local script_directory = script:parent()
end

getscriptpath()