--!nolint LocalShadow

local fs = require("@std/fs")
local mlua = require("@interop/mlua")

local function findfile()
	local file = fs.find("./src/main.rs").file
	assert(file ~= nil, "where file?")
end

local function finddir()
	local dir = fs.find("./src").dir
	assert(dir ~= nil, "where dir?")
end

local function nonexistentfile()
	local file = fs.find("./starmfeta.txt").file
	assert(file == nil, "why does this file exist?")
	assert(fs.find("./starmfeta.txt"):exists() == false, "exists fnn broke?")
end

local function nonexistentdir()
	local dir = fs.find("./starstar")
	assert(dir.dir == nil, "why does this dir exist?")
	assert(dir:exists() == false, "why does this dir exist?")
end

local function unwrapfile()
	local file = fs.find("./src/main.rs"):unwrap_file()
	assert(file ~= nil, "this is impossible otherwise where's my error?")
	local s, f = pcall(function()
		return fs.find("./src"):unwrap_file()
	end)
	assert(mlua.iserror(f) == true, "why no error?")
	assert(tostring(f):match("a non%-file"), "weird error message?")
end

local function unwrapdir()
	local dir = fs.find("./src"):unwrap_dir()
	assert(dir ~= nil, "this is impossible otherwise where's my error?")
	local s, f = pcall(function()
		return fs.find("./Cargo.toml"):unwrap_dir()
	end)
	assert(mlua.iserror(f) == true, "why no error?")
	assert(tostring(f):match("a non%-dir"), "weird error message?")
end

local function retryfile()
	-- TODO: fix path.join it's still borkey
	local newscriptpath = "/" .. fs.path.join(script:parent(), "newscript.luau")
	local newscript = fs.find(newscriptpath)
	assert(newscript.file == nil and newscript:exists() == false, "it's not made yet or is it??")
	fs.writefile(newscriptpath, `print("hi from seal")`)
	local ns2 = newscript:retry_file()
	assert(newscript.file ~= nil, "okay it should be there now???")
	assert(ns2 ~= nil, "where are ya")
	fs.removefile(newscriptpath)
end

findfile()
finddir()
nonexistentfile()
nonexistentdir()
unwrapfile()
unwrapdir()
retryfile()