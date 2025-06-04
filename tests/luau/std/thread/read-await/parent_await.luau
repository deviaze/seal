local thread = require("@std/thread")

local data = {
	jackdotink = true,
	some = {
		nested = { "data", "is", "cool" },
		other = {
			data = 1,
		}
	},
}

local child_thread = thread.spawn {
	path = "./child_await.luau",
	data = data,
}

print("PARENT: child is waiting for 1.25 seconds while we can do w/e we want")
-- do smth here

local response = child_thread:read_await() :: { message: string }
assert(response.message:match("I daresay"), "unexpected data received from child thread?")

print("PARENT: okay time to send child a buffer")
local buffy = buffer.fromstring("hello there")
child_thread:sendbytes(buffy)

child_thread:join()