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
			return nil
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

if env.os == "Windows" then
	local function pathparent()
		local cwd = path.cwd()
		local mainrs = path.join(cwd, "src", "main.rs")
		local src = path.parent(mainrs)
		assert(src ~= nil, "where srcdir??")
		assert(path.child(src) == "src", "cant path.child src?")
	end
	pathparent()
end

local function normalize()
	local tp1 = [[./hi/i/am/a/cat.txt]]
	assert(tp1 == path.normalize(tp1), "normalized simple path shouldn't change")

	local relative_without_backslashes = [[.\hi\i\am\cat.txt]]

	assert(
		path.normalize(relative_without_backslashes) == "./hi/i/am/cat.txt", 
		"path.normalize should always change relative paths to forward slashes"
	)

	local mixed_paths = [[./src\main.luau]]
	assert(path.normalize(mixed_paths) == "./src/main.luau", "mixed path 1 should be fixed to /")

	local windows_absolute = [[C:\Users\sealey\Desktop\Repositories\seal]]
	assert(path.normalize(windows_absolute) == windows_absolute, "windows absolute paths shouldn't be forward slashed")

	local mixed_windows_absolute = [[C:\Users\sealey\Desktop\Repositories\seal/main.luau]]
	assert(
		path.normalize(mixed_windows_absolute) == [[C:\Users\sealey\Desktop\Repositories\seal\main.luau]],
		"mixed windows absolute paths should be normalized to backslash"
	)

	local with_repeated_slashes = [[C:\Users\sealey\\hehe\/weird\slashes.hmm]]
	assert(
		path.normalize(with_repeated_slashes) == [[C:\Users\sealey\hehe\weird\slashes.hmm]],
		"repeated slashes not normalized?"
	)
end

normalize()
