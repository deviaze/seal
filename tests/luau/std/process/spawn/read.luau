local env = require("@std/env")
local path = require("@std/fs/path")
local process = require("@std/process")
local str = require("@std/str")

local function spawn_long_waiter(): process.ChildProcess
	return process.spawn {
		program = env.executable_path,
		args = { path.join(script:parent(), "long_waiter.luau") }
	}
end

local function makesurelinesdoesntbreak()
	local child = spawn_long_waiter()

	for line in child.stdout:lines() do
		-- print(line)
		break
	end

	-- print(child.stdout:read(nil, 2))
end

makesurelinesdoesntbreak()

local function basicread()
	local child = spawn_long_waiter()
	local text = child.stdout:read()
	assert(text == "hehe\n", "text not match hehe\\n")
	print(text)
end

basicread()

local function readloop()
	local child = process.spawn {
		program = env.executable_path,
		args = { path.join(script:parent(), "longrunning.luau") },
	}

	local result = ""
	while true do
		local stuff = child.stdout:read()
		if stuff then
			result ..= stuff
		end
		if str.endswith(result, "Stopping\n") then
			break
		end
	end
	print(result)
end

readloop()

local function mixingreads()
	local stdout = spawn_long_waiter().stdout
	
	local first = stdout:read_exact(1)
	assert(first == "h", "first should be h")
	local readnormally = stdout:read()
	print(readnormally)
end

mixingreads()