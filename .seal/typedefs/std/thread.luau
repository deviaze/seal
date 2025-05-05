--[=[
Run Luau code in parallel threads and communicate between threads via message passing.

## Usage
```luau
local thread = require("@std/thread")

local handle = thread.spawn {
	path = "./child.luau", -- note these paths are relative unlike std/fs paths
	data = {}, -- you can optionally pass in startup data to use in the other thread
}

-- receive data from your thread using handle:read and handle:read_await
local data = handle:read_await()
while thread.sleep(20) do
	local data = handle:read()
	if data then
		print(data)
	end
end
-- send data to your child thread using handle:send and handle:sendbytes
handle:send({ info = "hi" })

-- don't forget to join your threads before your program exits!!
handle:join()
```
]=]
local thread = {}

export type ThreadHandle = {
	--- joins the child thread to the main thread! don't forget to `:join` your handles lest you want runaway threads!!
	join: (self: ThreadHandle) -> (),
	--- serializes and sends arbitrary data through a channel. note that non-string data is serialized to json for transport.
	send: <D>(self: ThreadHandle, data: D | string) -> (),
	--- sends a buffer to the child thread without any serialization; on its own channel
	sendbytes: <D>(self: ThreadHandle, data: buffer) -> (),
	--- attempts to read from the main channel without yielding; returns the data sent on the channel if data is found, otherwise returns nil
	read: <D>(self: ThreadHandle) -> D?,
	--- reads from the main channel and yields until data is found
	read_await: <D>(self: ThreadHandle) -> D,
	--- reads from the bytes channel and returns a buffer if data is found, otherwise returns nil
	readbytes: (self: ThreadHandle) -> buffer?,
	--- reads from the bytes channel and yields until data is found
	readbytes_await: (self: ThreadHandle) -> buffer,
}
--[=[
Spawns and runs Luau code in a new Luau VM in a new (Rust) thread.

## Usage
```luau
-- main.luau
local thread = require("@std/thread")

local urls = { 
	"https://sealfinder.net/api/random", 
	"https://example.com/endpoint",
}

local threadpool: { thread.ThreadHandle } = {}
for _, url in urls do
	local handle = thread.spawn {
		path = "./web_get.luau",
		data = { url = url },
	}
	table.insert(threadpool, handle)
end

while true do
	for index, handle in threadpool do
		local response = handle:read()
		if response then
			print(response)
			handle:join()
			table.remove(threadpool, index)
		end
	end
end

-- web_get.luau
if channel then -- make sure we're in a child thread
	local http = require("@std/net/http")
	local response = http.get {
		url = channel.data.url,
	}
	channel:send(response)
end
```
]=]
function thread.spawn(spawn_options: { path: string, data: any? }): ThreadHandle
	return nil :: any
end

--[=[
Literally the same as `time.wait`, except in milliseconds.
]=]
function thread.sleep(milliseconds: number): true
	return nil :: any
end

return thread