--!nolint ImportUnused
local fs = require("@std/fs")
local env = require("@std/env")
local process = require("@std/process")

local basic_requires_dir = fs.dir.from(script:parent())

local function dirwithinit()
	local canweinitdir = fs.dir.ensure(basic_requires_dir:join("canweinit"))
		:add_file("init.luau", 'return "wecaninit"')
	local success, result = pcall(function()
		return (require :: any)("./canweinit")
	end)
	assert(result == "wecaninit", "we can't require directory with init.luau?")
	canweinitdir:remove()
end

dirwithinit()

local function relative()
	local t0 = os.clock()
	local relative_data = (require :: any)("./relative_data")
	local t1 = os.clock()

	assert(relative_data.relative == "yes very", "relative require didnt work")
	assert((t1 - t0) < 0.0035, "basic relative require took too long")
end

relative()

local function parent()
	local some_data_src = [[
		return {
			d = 5,
			b = {
				humans = 1,
				cats = 2,
			},
		}
	]]
	local parent_path = fs.path.parent(basic_requires_dir.path) :: string
	local some_data_path = fs.path.join(parent_path, "some_data.luau")
	fs.writefile(some_data_path, some_data_src)
	local parent_data = (require :: any)("../some_data")
	assert(parent_data.b and parent_data.b.cats, "parent require didnt work")
	fs.removefile(some_data_path)
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

-- best we can do is not infinite yield
local function circular()
	local circulars = fs.dir.ensure(basic_requires_dir:join("circulars"))
		:add_file("circular.luau", 'return require("./callmeback")')
		:add_file("callmeback.luau", 'local circular = require("./circular")')

	local start_time = os.clock()
	-- we don't want to infinite yield (lune behavior)
	local result = process.run {
		program = env.executable_path,
		args = { circulars:expect_file("callmeback.luau").path }
	}
	assert(os.clock() - start_time < 1, "oh no are we infinite yielding?")
	circulars:remove()
end

circular()

local function showshelpermessage()
	local s, f = pcall(function()
		local relative_data = (require :: any)("relative_data")
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

return {}