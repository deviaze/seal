local fs = require("@std/fs")
local env = require("@std/env")

local function simplepaths()
	assert(fs.path.join(".", "src", "main.rs") == "./src/main.rs", "simple paths broke?")
end

simplepaths()

local function slashesgetstripped()
	local p = `/{fs.path.join("cats", "/dogs")}/`
	local slashed = fs.path.join("./", p)
	assert(slashed == "./cats/dogs", "slashes didnt get stripped?")
end

slashesgetstripped()

if env.os == "Windows" then
	local function relativebackslashpaths()
		local p = fs.path.join(".\\", "src", "main.rs")
		assert(p == ".\\src\\main.rs", "windows backslash relative paths not work?")
	end
	relativebackslashpaths()
end

local function appendcwd()
	local testpath = fs.path.join(fs.path.cwd(), "tests")
	assert(fs.path.exists(testpath), `where the ./tests at? (path: {testpath})`)
end

appendcwd()

local function winabsolutes()
	local winpath = fs.path.join("C:", "Users", "Me", "Downloads", "meow.jpeg")
	assert(winpath == [[C:\Users\Me\Downloads\meow.jpeg]], "winpath not match")
end

winabsolutes()

local function absolutes()
	local firstcomproot = fs.path.join("/usr", "home")
	assert(
		firstcomproot == "/usr/home",
		"weird first component /usr got stripped?"
	)
	local usr_path = fs.path.join("/", "usr", "home")
	assert(
		usr_path == "/usr/home",
		`should be /usr/home, got: {usr_path}`
	)
end

absolutes()

local function uncsplease()
	local server_01 = fs.path.join([[\\]], "server01", "public")
	assert(
		server_01 == [[\\server01\public]],
		"server01 unc broke?"
	)
	local server_utf8 = fs.path.join([[\\server04]], "共享", "项目\\计划.docx")
	assert(
		server_utf8 == [[\\server04\共享\项目\计划.docx]],
		"server_utf8 broke?"
	)
	local questionmarkunc = fs.path.join([[\\]], "?", [[\UNC]], "server06", [[\downloads]], "file.txt")
	assert(
		questionmarkunc == [[\\?\UNC\server06\downloads\file.txt]],
		"questionmarkunc not match?"
	)
end

uncsplease()

local function unrelatives()
	local unrel = fs.path.join("cats", "dogs", "animal.txt")
	assert(
		unrel == "cats/dogs/animal.txt",
		"simple unrelative path component should work without prepending any relative"
	)
end

unrelatives()

local function tildeing()
	local localdotbin = fs.path.join("~", ".local", "bin")
	assert(
		localdotbin == "~/.local/bin",
		"localdotbin not binning"
	)
	local windowslocaldotbin = fs.path.join("~\\", ".local", "bin")
	assert(
		windowslocaldotbin == [[~\.local\bin]],
		"windows local bin not binning (not \\ separator?)"
	)
end

tildeing()