local process = require("@std/process")
local env = require("@std/env")
local path = require("@std/fs/path")
local str = require("@std/str")

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
	assert(
		typeof(meowres) == "string" and (not meowres:match("meow")), 
		"unexpected meowres"
	)
	assert(
		str.startswith(tostring(os.clock() - start_time), "0.15"),
		"read_until with 0.15s timeout should be very close to 0.15s"
	)
end

interruptreadwithtimeout()