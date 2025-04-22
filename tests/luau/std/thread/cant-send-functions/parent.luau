--!nonstrict
local thread = require("@std/thread")
local time = require("@std/time")

local child_handle = thread.spawn({
	path = "./child.luau",
})

child_handle:send({
	stuff = "aa"
})

time.wait(0.125)

print(child_handle:read())

local try = require("@std/testing/try")
try(function()
	child_handle:send(function() end)
end):expect_err("thread.send expected")

try(function()
	child_handle:send({
		cats = 2,
		meow = function(n: number)
			return string.rep("meow", n)
		end,
	})
end):expect_err("cannot serialize <function>")


child_handle:join()