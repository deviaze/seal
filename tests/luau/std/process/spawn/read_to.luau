local process = require("@std/process")
local env = require("@std/env")
local path = require("@std/fs/path")
local str = require("@std/str")

local cheese = require("@tests/cheese")

local function spawn_longrunning(): process.ChildProcess
	local child = process.spawn {
		program = env.executable_path,
		args = { path.join(script:parent(), "longrunning.luau"), "60", "50" }
	}
	return child
end

local function onstart()
	local child = spawn_longrunning()
	local res = child.stdout:read_to("..", false)
	assert(typeof(res) == "string" and res == "Starting", ":read_to should be Starting here")
	local dot = child.stdout:read_to("\n")
	assert(dot == ".", "Should get the . after Starting.. and before \\n")
end

onstart()

local function readtoend()
	local child = spawn_longrunning()
	-- passing "" search term is equivalent to reading til `ChildProcess` exits
	local result = child.stdout:read_to("")
	assert(
		typeof(result) == "string"
		and str.endswith(result, "Stopping\n"),
		"Stopping\\n isn't the last thing in result after read to end??"
	)
end

readtoend()

local function interruptreadtowithtimeout()
	local child = process.spawn {
		program = env.executable_path,
		args = { path.join(script:parent(), "long_waiter.luau") }
	}
	
	local start_time = os.clock()
	-- long_waiter prints "meow" but only after 0.75 seconds, so if we read with a timeout of 0.15
	-- we shouldn't expect to see "meow", we should see `nil`
	local meowres = child.stdout:read_to("meow", true, 0.15)
	assert(meowres == nil, "meowres should be nil")
	assert(
		str.startswith(tostring(os.clock() - start_time), "0.15"),
		"read_to with 0.15s timeout should be very close to 0.15s"
	)
end

interruptreadtowithtimeout()

local function interruptedreadallowpartial()
	local child = process.spawn {
		program = env.executable_path,
		args = { path.join(script:parent(), "long_waiter.luau") }
	}

	local start_time = os.clock()
	-- long_waiter prints "meow" after 0.75 seconds, so if we interrupt with a 0.15 timeout
	-- and allow partial reads, we should get hehe\n instead
	local heheres = child.stdout:read_to("meow", true, 0.15, true)
	assert(heheres == "hehe\n", "heheres should be hehe\\n")
	assert(
		str.startswith(tostring(os.clock() - start_time), "0.15"),
		"read_to with 0.15s timeout should be very close to 0.15s"
	)
	-- now that's been read, we should be able to easily get meowres
	local start_2_time = os.clock()
	local meowres = child.stdout:read_to("meow", true)
	assert(typeof(meowres) == "string" and (meowres:match("meow")), "meowres should be meow now and we shouldn't have the trailing \\n included")
	if env.os == "Linux" then
		assert(
			str.startswith(tostring(os.clock() - start_2_time), "1.8"),
			"read_to should now find \"meow\" in around 1.8something seconds since the first 0.15 seconds were already used"
		)
	else
		-- can't really make any guarantees here :/
	end
end

cheese.retry(interruptedreadallowpartial, 3)