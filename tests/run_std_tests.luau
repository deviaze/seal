local fs = require("@std/fs")
local process = require("@std/process")
local colors = require("@std/colors")

local passed_tests: { string } = {}
local failed_tests: { { file: string, err: string } } = {}

local ignore_files = {
	"./tests/luau/std/net/server/serve.luau",
	"./tests/luau/std/thread/get-threads/send_request.luau",
	"./tests/luau/std/net/server/client.luau",
	"./tests/luau/std/thread/conc_1.luau",
}

local tests_with_stdin = {"io/init", "stdin", "input"}

local test_files: { string } = {} do
	local file_list = fs.listdir("./tests/luau/std", true)
	for _, path in file_list do
		if (not path:match("%.luau$")) or table.find(ignore_files, path) then
			continue
		end
		table.insert(test_files, path)
	end
end

local function _serial()
	for _, path in test_files do
		print(`starting {path}`)

		local result = process.run({
			program = "./target/debug/seal",
			args = { path },
		})
		if result.ok then
			table.insert(passed_tests, path)
		else
			table.insert(failed_tests, {
				file = path,
				err = result.stderr,
			})
		end
	end

	print(`Tests passed: {#passed_tests}\nTests failed: {#failed_tests}`)

	for index, test in failed_tests do
		print(`{index}:\n    {test.file}: {test.err}`)
	end
end

type Stream = index<process.ChildProcess, "stdout"> | index<process.ChildProcess, "stderr">

local function read_until(stream: Stream, hit: string?): string?
	local result = ""
	local token = ""
	local current: string? = stream:read(1)
	if current then
		result ..= current :: string
		while current ~= nil do
			current = stream:read(1)
			if hit and #token < #hit then
				token ..= current :: string
				if token == hit then
					break
				end
			elseif hit and #token >= #hit and token ~= hit then
				token = ""
			end
			if current then
				result ..= current
			end
		end
		return result
	else
		return nil
	end
end

local function parallel()
	local results: { { path: string, ok: boolean, err: string } } = {}
	local handles: { [string]: process.ChildProcess } = {}
	for _, test_path in test_files do
		local child = process.spawn {
			program = "./target/debug/seal",
			args = { test_path },
		}
		handles[test_path] = child
	end

	local start_time = os.clock()
	local tests_passed = 0
	local tests_failed = 0

	while (os.clock() - start_time) < 20 do -- 20 seconds
		for path, handle in handles do
			if not handle:alive() then
				local err: string? = read_until(handle.stderr)
				if err and err:match("ERR") then
					print(` ❌ {colors.red("test failed")}: {path}`)
					tests_failed += 1
					table.insert(results, {
						path = path,
						ok = false,
						err = err,
					})
				else
					tests_passed += 1
					print(` {if tests_passed < 10 then " " else ""}{tests_passed} {colors.yellow("test successful")}: {path}`)
					table.insert(results, {
						path = path,
						ok = true,
						err = "",
					})
				end
				handles[path] = nil
			else
				-- some tests need user input
				for _, p in tests_with_stdin do
					if path:match(p) then
						handle.stdin:write(`{math.random(1, 223)}\n`)
					end
				end
			end
		end
		if #results == #test_files then
			break
		end
	end

	for path, handle in handles do
		if handle:alive() then
			print(`{(colors.magenta("time limit exhausted"))}: {path}`)
			handle:kill()
			tests_failed += 1
			table.insert(results, {
				ok = false,
				path = path,
				err = "time limit exhausted",
			})
		end
	end

	p("")

	print(`{colors.yellow("passed")}: {tests_passed}`)
	print(`{colors.red("failed")}: {tests_failed}`)

	local current_fail_index = 0

	for _, result in results do
		if result.ok then
			continue
		end
		current_fail_index += 1
		print(`{current_fail_index}    {result.path}: {result.err}`)
	end
end

-- local t0 = os.clock()
-- serial()
-- print(os.clock() - t0) -- 45 seconds

local t1 = os.clock()
parallel()

print(`seconds: {colors.bold.blue(tostring(os.clock() - t1))}`)