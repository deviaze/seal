local err = require("@std/err")
local fs = require("@std/fs")

local function withmessage()
	local e = err.message("meow")
	assert(typeof(e) == "error", "wrapped error should be made of error")
end

withmessage()

local function formatting()
	local _, res = pcall(function()
		return fs.readfile("./neowi.rs")
	end)
	assert(typeof(res) == "error", "reading bad file should error for err.format formatting")
	local formatted = err.format(res)
	print(`err [[{formatted}]]`)

	local wrapped = err.wrap("cats are cool")
	err.throw(wrapped)
end

formatting()