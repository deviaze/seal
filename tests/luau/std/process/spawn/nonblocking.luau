local env = require("@std/env")
local process = require("@std/process")

local folder_path = "./tests/luau/std/process/spawn/"
local longrunning_path = `{folder_path}/longrunning.luau`

local seal_path = env.executable_path

local function test_reads()
	local child = process.spawn {
		program = seal_path,
		args = { longrunning_path },
	}
	
	while true do
		local r = child.stdout:read(24)
		if r then
			print(r)
		end
		if not child:alive() then
			print("exiting")
			break
		end
	end
end

local function test_lines()
	local child = process.spawn {
		program = seal_path,
		args = { longrunning_path },
	}
	
	for line in child.stdout:lines() do
		print(line)
	end	
end

local function test_stdin()
	local stdin_path = `{folder_path}/has_stdin.luau`
	local child = process.spawn {
		program = seal_path,
		args = { stdin_path }
	}
	child.stdin:write("yes\n")
	for line in child.stdout:lines() do
		print(line)
	end
end

test_reads()
test_lines()
test_stdin()