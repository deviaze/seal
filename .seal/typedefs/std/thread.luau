	--[=[
	Run Luau code in parallel in a new VM and communicate between threads via message passing.

	This allows for structured parallelism that you can use for both multiprocessing and as
	a replacement for async/task library usage, although with a little more boilerplate in the latter usecase.

	Threads can be spawned via `thread.spawn`, communicated with on the *regular* or *bytes* channels with the `send*` and `read*` apis, and joined back
	to the parent thread with `thread.join`.

	### Channels

	Each thread come with 2 channels to communicate with its parent thread:

	On the *regular* channel, messages can be either data tables (tables consisting of json-serializable objects)--seal automatically
	serializes and deserializes data tables for simplicity and ergonomics--or strings.

	On the *bytes* channel, data can be sent and received with `buffer`s without any serialization overhead.

	Each channel has a queue; by default the *regular* channel's queue capacity is 12 messages and the *byte* channel 24,
	although this is configurable with `thread.spawn`'s `ThreadSpawnOptions`. Reading a message will pop it from the queue.

	## Usage
	```luau
	-- parent.luau
	local thread = require("@std/thread")

	local handle = thread.spawn {
		path = "./child.luau", -- note these paths are relative like luau requires and unlike std/fs paths
		data = { urls = urls }, -- you can optionally pass in startup data to use in the other thread
	}

	-- receive data from your thread using handle:read and handle:read_await
	local data = handle:read_await()
	while thread.sleep(20) do
		local data = handle:read()
		if typeof(data) == "table" then
			print(data)
		else
			break
		end
	end
	-- send data to your child thread using handle:send and handle:sendbytes
	handle:send("hi")

	-- don't forget to join your threads before your program exits!!
	handle:join()

	-- child.luau
	if channel then -- channel is a global that exists in child threads and can be used to communicate with the parent thread
		local urls = (channel.data :: { urls: { string } }).urls
		channel:send("first")
		for _, url in urls do
			local result = callapi(url)
			channel:send(result)
		end
		channel:send("done")
	end

	```
]=]
local thread = {}

export type JsonSerializableTable = { [unknown]: unknown }

export type ThreadHandle = {
	--- the name of your thread (defaults to a petname if not provided)
	read name: string,
	-- ThreadHandle:join()
	--[=[
		Joins the child thread back to the main thread; don't forget to join your handles lest you want runaway threads!

		Errors if the thread has already been joined or somehow disappeared.
	]=]
	join: (self: ThreadHandle) -> (),
	-- ThreadHandle:send(data)
	--[=[
		Serializes and sends data to the child thread on the regular channel. Data can either be a string or a JsonSerializableTable; table data is serialized to json for transport
		and automatically deserialized when received by :read methods.

		If the channel is full, blocks the current thread until the channel isn't full anymore. 
		If you want to not block the current thread, use `try_send` instead.

		Errors if the channel has somehow become disconnected or provided data isn't json-serializable.
	]=]
	send: (self: ThreadHandle, data: JsonSerializableTable | string) -> (),
	-- ThreadHandle:try_send(data)
	--[=[
		Try to send data to the child thread on the regular channel with the same semantics as `ThreadHandle:send`, 
		except doesn't block if the channel is already full, and doesn't throw an error if the channel is disconnected.

		Returns two values: success and result.
		- `result == "Disconnected"` means that the channel was disconnected and either the Sender or Receiver no longer exists.
		This is usually caused by trying to send a message to a thread that's already been joined or exited.
		- `result == "Full"` means that the channel's queue is full and no more new messages can be sent until the other side starts reading from the queue.
	]=]
	try_send: (self: ThreadHandle, data: JsonSerializableTable | string) -> (boolean, "Sent" | "Disconnected" | "Full"),
	-- ThreadHandle:sendbytes(data: buffer)
	--[=[
		Sends a buffer on the bytes channel, blocking the current thread if the channel is full.

		Errors if the channel has somehow become disconnected.	
	]=]
	sendbytes: (self: ThreadHandle, data: buffer) -> (),
	-- ThreadHandle:try_sendbytes(data: buffer)
	--[=[
		Try to send data on the bytes channel with the same semantics as `ThreadHandle:sendbytes`, 
		except doesn't block if the channel is already full, and doesn't throw an error if the channel is disconnected.

		Returns two values: success and result.
		- `result == "Disconnected"` means that the channel was disconnected and either the Sender or Receiver no longer exists.
		This is usually caused by trying to send a message to a thread that's already been joined or exited.
		- `result == "Full"` means that the channel's queue is full and no more new messages can be sent until the other side starts reading from the queue.
	]=]
	try_sendbytes: (self: ThreadHandle, data: buffer) -> (boolean, "Sent" | "Disconnected" | "Full"),
	-- ThreadHandle:read()
	--[=[
		Read a message from the regular channel without blocking the current thread.

		Errors if the channel has somehow become disconnected.
	]=]
	read: (self: ThreadHandle) -> JsonSerializableTable? | string?,
	-- ThreadHandle:read_await()
	--[=[
		Read a message from the regular channel, blocking until the next message is available.

		Errors if the channel has somehow become disconnected.
	]=]
	read_await: (self: ThreadHandle) -> JsonSerializableTable | string,
	-- ThreadHandle:readbytes()
	--[=[
		Read a message from the bytes channel without blocking the current thread.

		Errors if the channel has somehow become disconnected.
	]=]
	readbytes: (self: ThreadHandle) -> buffer?,
	-- ThreadHandle:readbytes_await()
	--[=[
		Read a message from the bytes channel, blocking until the next message is available.

		Errors if the channel has somehow become disconnected.
	]=]
	readbytes_await: (self: ThreadHandle) -> buffer,
}

export type ThreadSpawnOptions = {
	--- Name your thread to quickly identify which one it is; if not provided a default alliterative petname will be provided instead.
	name: string?,
	--- Path to your source file you want to run in the separate thread, relative to the current file (not cwd).
	path: string?,
	--- Source code to evaluate; recommend passing a path instead.
	src: string?,
	--- Optional data you want to provide to your thread at startup; accessible with `channel.data` in the child thread.
	data: JsonSerializableTable?,
	--- Override the queue capacity of your thread's regular and bytes channels.
	capacity: {
		--- default is 12
		regular: number?,
		--- default is 24
		bytes: number?,
	}?
}

	--[=[
	Spawns a new Rust Thread running Luau code in a new Luau VM.

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
function thread.spawn(spawn_options: ThreadSpawnOptions): ThreadHandle
	return nil :: any
end

--[=[
	Literally the same as `time.wait`, except in milliseconds.
]=]
function thread.sleep(milliseconds: number): true
	return nil :: any
end

return thread