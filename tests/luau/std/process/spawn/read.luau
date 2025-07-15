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

local longrunning_path = path.join(script:parent(), "longrunning.luau")

local function spawn_longrunning(): process.ChildProcess
    local child = process.spawn {
        program = env.executable_path,
        args = { longrunning_path, "85", "50" },
    }
    return child
end

local function makesurelinesdoesntbreak()
	local child = spawn_long_waiter()

	for line in child.stdout:lines() do
		-- print(line)
		break
	end

end

makesurelinesdoesntbreak()

local function basicread()
	local child = spawn_long_waiter()
	local text = child.stdout:read(0.15) -- explicitly specify duration here to prevent inconsistent test failure
	assert(text == "hehe\n", "text not match hehe\\n")
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
	assert(str.endswith(result, "Stopping\n"))
end

readloop()

local function generalizedstreamed()
    local child = spawn_longrunning()
    local i = 0
    for message in child.stdout do
        i += 1
        if i == 1 then
            assert(string.match(message, "Starting..."))
        end
    end
end

generalizedstreamed()

local function mixingreads()
	local stdout = spawn_long_waiter().stdout
	local first = stdout:read_exact(1, 0.5) -- explicitly specify duration to prevent inconsistent test failure
	assert(first == "h", "first should be h")
	local readnormally = stdout:read()
	assert(readnormally == "ehe\n")
end

mixingreads()