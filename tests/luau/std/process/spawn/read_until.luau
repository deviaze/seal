local process = require("@std/process")
local env = require("@std/env")
local path = require("@std/fs/path")

local child = process.spawn {
	program = env.executable_path,
	args = { path.join(script:parent(), "longrunning.luau") }
}

local function startinguntil()
	local res = child.stdout:read_until("..", nil)
	assert(typeof(res) == "string" and res == "Starting..", ":read_until should be Starting.. here")
end

startinguntil()

local function interruptreadwithtimeout()
	-- this mf keeps blocking after reading 'hehe' and can't get timeout
	-- to interrupt the reading process before 'meow' is printed
	local child2 = process.spawn {
		program = env.executable_path,
		args = { path.join(script:parent(), "long_waiter.luau") }
	}
	
	local start_time = os.clock()
	local meowres = child2.stdout:read_until("meow", 0.15)
	print(meowres)
	print(os.clock() - start_time)
end

interruptreadwithtimeout()