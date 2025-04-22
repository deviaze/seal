-- requiring std libraries works (it's an alias ig)
local _fs = require("@std/fs")

local function aliased()
	local relative_but_not_relatively = (require :: any)("@tests/globals/require/basic-requires/relative_data")
	assert(relative_but_not_relatively.relative == "yes very", "alias required file returned unexpected result")
end

aliased()

local function unabletoextractalias()
	local s, f = pcall(function()
		local impossible = (require)("@/hi")
	end)
	assert(s == false, "extracting alias was not supposed to succeed")
	assert(tostring(f):match("couldn't extract require alias"), "received different error when trying to extract alias")
end

unabletoextractalias()

local function nonexistentalias()
	local s, f = pcall(function()
		local impossible = (require)("@hi/hi")
	end)
	assert(s == false, "requiring nonexisting alias was not supposed to succeed")
	assert(tostring(f):match("alias 'hi' was not found in the .luaurc"))
end

nonexistentalias()
