local thread = require("@std/thread")
local time = require("@std/time")
local process = require("@std/process")
local format = require("@std/io/format")

local handle = thread.spawn({
	path = "./child_thread.luau"
})
local t = 0
while time.wait(1) do
	print(`{t} seconds...`)
	t += 1
	if t % 2 == 0 then
		handle:send {
			t = t,
			content = "well"
		}
		local response = handle:read() :: {
			t: number,
			content: string
		}?

		if response then
			if response.t > 6 then
				print("done")
				handle:send {
					t = t,
					content = "break"
				}
				handle:join()
				process.exit()
			else
				print(`child says: {format(response)}`)
			end
		end

	end
end

handle:join()