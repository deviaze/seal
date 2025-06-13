--!nonstrict
--!nolint LocalShadow
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

local s, err = pcall(function()
	child_handle:send(function() end)
end)

assert(
	s == false
	and typeof(err) == "error"
	and tostring(err):match("expected data to be"),
	"sending function as data should fail"
)

local s, err = pcall(function()
	child_handle:send {
		cats = 2,
		meow = function(n: number)
			return string.rep("meow", n)
		end,
	}
end)

assert(
	s == false
	and typeof(err) == "error"
	and tostring(err):match("cannot serialize <function>"),
	"expected to not be able to serialize function; error message doesn't match?"
)

child_handle:join()