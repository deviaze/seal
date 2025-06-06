--!nolint ImportUnused
local fs = require("@std/fs")
local env = require("@std/env")
local process = require("@std/process")

local basic_requires_dir = fs.dir.from(script:parent())

local function dirwithinit()
	local canweinitdir = fs.dir.ensure(basic_requires_dir:join("canweinit"))
		:add_file("init.luau", 'return "wecaninit"')
	local success, result = pcall(function(): any
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
	local s, f = pcall(function(): nil
		(require)("./invalid_file")
		return nil
	end)
	fs.removefile(invalid_file_path)
	assert(tostring(f):match("Expected identifier when parsing variable name"), "requiring an invalid file doesn't produce the expected error")
end

invalidfile()

-- best we can do is not infinite yield
--/sealignore: flakey asf test
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
	if env.os ~= "Windows" then
		-- flakey test on windows probably file locks or smth
		-- assert(os.clock() - start_time < (
		-- 	if env.os ~= "Windows" then 1.25 else 4 -- windows is slow potato of a process runner
		-- ), "oh no are we infinite yielding?")
	end
	circulars:remove()
end

circular()

local function showshelpermessage()
	local s, f = pcall(function()
		local relative_data = (require :: any)("relative_data")
		return nil
	end)
	assert(tostring(f):match("does not start with"), "issue showing prefix helper message")
end

showshelpermessage()

local function filenotfound()
	local s, f = pcall(function()
		local notfound = (require)("./notfound")
		return nil
	end)
	assert(s == false, "requiring filenotfound was not supposed to succeed")
	assert(tostring(f):match("not found"), "not getting not found when requiring a file that should not be found")
end

filenotfound()

local function filethaterrors()
	local iwillbreakpath = script:parent() .. "/iwillbreak.luau"

	fs.writefile(iwillbreakpath, 'error("hi")')
	
	local s, f = pcall(function()
		(require)("./iwillbreak")
		return nil
	end)

	assert(s == false, "iwillbreak.luau did not actually break?")
	assert(tostring(f):match('message: "hi"'), "iwillbreak broke with an unexpected error message")
	
	if fs.file(iwillbreakpath) then
		fs.removefile(iwillbreakpath)
	end
end

filethaterrors()

return {}
