--!nonstrict
local thread = require("@std/thread")
local time = require("@std/time")

local handle = thread.spawn({
	path = "./conc_1.luau"
})

print("first")

local t = 0
repeat
	time.wait(0.25)
	t += 1
	print("hm")
until t == 20

local s, f = pcall(function()
	handle:join()
end)

assert(s == false, "thread.spawn should have errored")


print("two")