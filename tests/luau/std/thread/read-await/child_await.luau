--!nonstrict
local time = require("@std/time")

if channel then
	local data = channel.data :: any
	print(`CHILD: recv: {table.concat(data.some.nested)}`)
	time.wait(1.25)
	channel:send({
		message = "I daresay data is cool too"
	})
	local buffy = channel:readbytes_await()
	local recv = buffer.readstring(buffy, 0, 5)
	assert(recv == "hello", "should be the 'hello' in 'hello there'")
end
