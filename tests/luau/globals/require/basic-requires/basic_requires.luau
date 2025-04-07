--!nolint ImportUnused
local fs = require("@std/fs")

local function relative()
	local t0 = os.clock()
	local relative_data = require("./relative_data")
	local t1 = os.clock()
	assert(relative_data.relative == "yes very", "relative require didnt work")
	assert((t1 - t0) < 0.0025, "basic relative require took too long")
end

relative()

local function parent()
	local parent_data = require("../some_data")
	assert(parent_data.b and parent_data.b.cats, "parent require didnt work")
end

parent()

local basic_requires_path = "./tests/luau/globals/require/basic-requires"

local function invalidfile()
	local invalid_file_path = script:parent() .. "/invalid_file.luau"
	fs.writefile(invalid_file_path, "local")
	local s, f = pcall(function()
		(require)("./invalid_file")
	end)
	fs.removefile(invalid_file_path)
	assert(tostring(f):match("Expected identifier when parsing variable name"), "requiring an invalid file doesn't produce the expected error")
end

invalidfile()

local function circular()
	local circular_path = script:parent().. "/circular.luau"
	fs.writefile(circular_path, 'return require("./basic_requires.luau")')
	local s, f = pcall(function()
		(require)("./tests/luau/globals/require/basic-requires/circular.luau") -- best we can do is make sure it doesn't infinite yield (lune's behavior)
	end)
	if fs.file(circular_path) then
		fs.removefile(circular_path)
	end
end

-- circular()

local function dirwithinit()
	local canweinitpath = script:parent() .. "/canweinit"
	if fs.dir(canweinitpath) then
		fs.removetree(canweinitpath)
	end

	fs.writetree(canweinitpath, {
		fs.file.build("init.luau", "return 'wecaninit'")
	})

	local recv = (require)("./canweinit")
	assert(recv == "wecaninit", "we cannot require dir with init.luau?")
	if fs.find(canweinitpath).dir then
		fs.removetree(canweinitpath)
	end
end

dirwithinit()

local function showshelpermessage()
	local s, f = pcall(function()
		local relative_data = require("relative_data")
	end)
	assert(tostring(f):match("Invalid require path format"), "issue showing the long 'Invalid require path format' helper message")
end

showshelpermessage()

local function filenotfound()
	local s, f = pcall(function()
		local notfound = (require)("./notfound")
	end)
	assert(s == false, "requiring filenotfound was not supposed to succeed")
	assert(tostring(f):match("require: file not found:"), "not getting file not found when requiring a file that should not be found")
end

filenotfound()

local function filethaterrors()
	local iwillbreakpath = script:parent() .. "/iwillbreak.luau"

	fs.writefile(iwillbreakpath, 'error("hi")')
	
	local s, f = pcall(function()
		(require)("./iwillbreak")
	end)

	assert(s == false, "iwillbreak.luau did not actually break?")
	assert(tostring(f):match('message: "hi"'), "iwillbreak broke with an unexpected error message")
	
	if fs.file(iwillbreakpath) then
		fs.removefile(iwillbreakpath)
	end
end

filethaterrors()
