-- requiring std libraries works (it's an alias ig)
local fs = require("@std/fs")
local output = require("@std/io/output")

local function aliased()
	local relative_but_not_relatively = (require :: any)("@tests/globals/require/basic-requires/relative_data")
	assert(relative_but_not_relatively.relative == "yes very", "alias required file returned unexpected result")
end

aliased()

local function unabletoextractalias()
	local s, f = pcall(function()
		local impossible = (require)("@/hi")
		return nil
	end)
	assert(s == false, "extracting alias was not supposed to succeed")
	assert(tostring(f):match("couldn't extract require alias"), "received different error when trying to extract alias")
end

unabletoextractalias()

local function nonexistentalias()
	local s, f = pcall(function()
		local impossible = (require)("@hi/hi")
		return nil
	end)
	assert(s == false, "requiring nonexisting alias was not supposed to succeed")
	assert(output.unformat(tostring(f)):match("alias 'hi' not found in the %.luaurc"), "unexpected alias not found message")
end

nonexistentalias()

local function aliaswithoutslashfile()
	local process_values = (require :: any)("@output_process_values")
	assert(process_values.simple_print ~= nil, "output_process_values.simple_print should be non-nil")
end

aliaswithoutslashfile()

local function aliaswithoutslashdir()
	local data_path = fs.path.join(".", "tests", "data")
	local data_dir = fs.dir.from(data_path)
		:add_file("init.luau", "return 'successful'")
	
	local res = (require :: any)("@data")
	assert(res == "successful", "why is requiring @data not successful?")
	fs.removefile(data_dir:join("init.luau"))
end

aliaswithoutslashdir()
