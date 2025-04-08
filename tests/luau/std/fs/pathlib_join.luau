local fs = require("@std/fs")
local env = require("@std/env")

local path = fs.path

local function simplepaths()
	assert(path.join(".", "src", "main.rs") == "./src/main.rs", "simple paths broke?")
end

simplepaths()

local function slashesgetstripped()
	local p = `/{path.join("cats", "/dogs")}/`
	local slashed = path.join("./", p)
	assert(slashed == "./cats/dogs", "slashes didnt get stripped?")
end

slashesgetstripped()

if env.os == "Windows" then
	local function relativebackslashpaths()
		local p = path.join(".\\", "src", "main.rs")
		assert(p == ".\\src\\main.rs", "windows backslash relative paths not work?")
	end
	relativebackslashpaths()
end

local function appendcwd()
	local testpath = path.join(path.cwd(), "tests")
	assert(path.exists(testpath), "where the ./tests at?")
end

appendcwd()