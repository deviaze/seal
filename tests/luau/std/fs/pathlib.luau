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
		path.normalize(relative_without_backslashes) ~= "./hi/i/am/cat.txt", 
		"path.normalize should not change relative paths to forward slashes when prepended with .\\"
	)

	local mixed_paths = [[./src\main.luau]]
	assert(path.normalize(mixed_paths) == "./src/main.luau", "mixed path 1 should be fixed to /")

	local windows_absolute = [[C:\Users\sealey\Desktop\Repositories\seal]]
	local windows_normalized = path.normalize(windows_absolute)
	assert(windows_normalized == windows_absolute, `windows absolute path shouldn't be modified; got: {windows_normalized}`)

	local mixed_windows_absolute = [[C:\Users\sealey\Desktop\Repositories\seal\./main.luau]]
	assert(
		path.normalize(mixed_windows_absolute) == [[C:\Users\sealey\Desktop\Repositories\seal\main.luau]],
		"mixed windows absolute paths should be normalized to backslash"
	)

	local with_repeated_slashes = [[C:\Users\sealey\\hehe\/weird\slashes.hmm]]
	assert(
		path.normalize(with_repeated_slashes) == [[C:\Users\sealey\hehe\weird\slashes.hmm]],
		"repeated slashes not normalized?"
	)

	local relative_parent = [[../../.\/meow.txt]]
	assert(
		path.normalize(relative_parent) == [[../../meow.txt]],
		"relative parent prefix shouldn't be stripped when we don't know what's in front"
	)

	local home_parent = [[~/meow/../..\\cats.jpeg]]
	assert(
		path.normalize(home_parent) == [[~/../cats.jpeg]], -- we actually don't want to expand ~ in path.normalize
		"weird ~ cats.jpeg not normalized to ~/../cats.jpeg"
	)

	local unc_paths = {
		[[\\server01\.//public]],
		[[\\fileserver\docs\reports\2025\summary.pdf]],
		[[\\datahub\shared folders\team alpha\plan.docx\.]],
		[[\\server02\admin$\config.ini]],
		[[\\192.168.1.100\media\video.mp4]],
		[[\\Server03\Finance\Q4\Budget.xlsx]],
		[[\\backup\archives\..\2025\August\week3\logs\system.log]],
		[[\\server04\共享\项目\计划.docx]],
		[[\\server05\downloads\]],
		[[\\?\UNC\server06\weird|path\file.txt]],
	}
	local expected_unc_paths = {
		[[\\server01\public]],
		[[\\fileserver\docs\reports\2025\summary.pdf]],
		[[\\datahub\shared folders\team alpha\plan.docx]],
		[[\\server02\admin$\config.ini]],
		[[\\192.168.1.100\media\video.mp4]],
		[[\\Server03\Finance\Q4\Budget.xlsx]],
		[[\\backup\2025\August\week3\logs\system.log]],
		[[\\server04\共享\项目\计划.docx]],
		[[\\server05\downloads]],
		[[\\?\UNC\server06\weird|path\file.txt]],
	}
	for index, p in unc_paths do
		local res = path.normalize(p)
		assert(
			res == expected_unc_paths[index], 
			`path.normalize {index}; {res} not matched {expected_unc_paths[index]}`
		)
	end

	local smol_path = "./a"
	assert(path.normalize(smol_path) == "./a", "smol path should be ./a without modification")

	assert(path.normalize(".") == ".", "cwd itself shouldn't pose issues")
end

normalize()

local function project()
	local proj = path.project()
	assert(fs.path.child(proj :: any) ~= nil, "why is project dir nil hmm")
end

project()