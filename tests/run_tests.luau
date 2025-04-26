--[=[
	To run all (luau) tests, run this file with `./target/debug/seal ./tests/run_tests.luau`
	note the cwd should be the seal repository
]=]

local fs = require("@std/fs")
local env = require("@std/env")
local colors = require("@std/colors")
local process = require("@std/process")
local output = require("@std/io/output")

local passed_tests: { string } = {}
local failed_tests: { { file: string, err: string } } = {}
local invalid_test_files: { [string]: string } = {}

local ignore_files = {
	"./tests/luau/std/net/server/serve.luau", -- manual test
	-- the below are not test files but are required by/imported by other tests
	"./tests/luau/std/thread/get-threads/send_request.luau",
	"./tests/luau/std/net/server/client.luau",
	"./tests/luau/std/thread/conc_1.luau",
	"./tests/luau/errors/another_module.luau",
	"./tests/luau/globals/require/basic-requires/relative_data.luau",
}

local tests_with_stdin = {"io/init", "stdin", "input"}

local test_files: { string } = {} do
	local file_list = fs.listdir("./tests/luau", true)
	for _, path in file_list do
		if (not path:match("%.luau$")) or table.find(ignore_files, path) then
			continue
		end
		table.insert(test_files, path)
	end
end

--- functions ignored with key being the function name
local ignored_functions: { [string]: { test_path: string, reason: string } } = {}
local assert_count = 0
do
	for _, test_path in test_files do
		local last_reason: string = ""
		local grab_next_line = false
		for n, line in fs.readlines(test_path) do
			if string.match(line, "^%s*assert%(") then -- excludes commented-out asserts
				assert_count += 1
				continue
			end
			if string.match(line, "%s*--/sealignore") then
				local found_reason = string.match(line, "%s*--/sealignore[:]*%s*([%w%s%p]+)$")
				if found_reason then
					last_reason = found_reason
					grab_next_line = true
					continue
				end
			end
			if grab_next_line then
				local function_name = string.match(line, "%s*local function ([%w_]+)%(")
				if function_name then
					ignored_functions[function_name] = {
						test_path = test_path,
						reason = last_reason,
					}
				end
				last_reason = ""
				grab_next_line = false
			end
		end
	end
end

local function _serial()
	for _, path in test_files do
		print(`starting {path}`)

		local result = process.run({
			program = env.executable_path,
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
-- TODO: add this to ChildProcess api itself
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

-- we want to ignore ignored tests
type IgnoredFunction = {
	name: string,
	reason: string,
}
local function check_ignored(test_path: string, err: string?): { IgnoredFunction }?
	local result: { IgnoredFunction } = {}
	for function_name, info in ignored_functions do
		if info.test_path ~= test_path then
			continue
		end
		if err and string.match(err, function_name) then
			-- if an ignored test case has asserts that fail, then it's an invalid test file because test cases after it won't be run
			local invalid_reason = `all asserts in ignored test case '{function_name}' need to be commented out`
			invalid_test_files[info.test_path] = invalid_reason
			print(`{colors.bold.red("  invalid test file:")} {info.test_path}\n    {colors.red(invalid_reason)}`)
		end
		table.insert(result, {
			name = function_name,
			reason = info.reason,
		})
	end
	return if #result > 0 then result else nil
end

local function parallel()
	local results: { { path: string, ok: boolean, err: string } } = {}
	local handles: { [string]: process.ChildProcess } = {}
	for _, test_path in test_files do
		local child = process.spawn {
			program = env.executable_path, -- usually ./target/debug/seal
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
					if not check_ignored(path, err) then
						print(` ❌ {colors.red("test failed")}: {path}`)
						tests_failed += 1
						table.insert(results, {
							path = path,
							ok = false,
							err = err,
						})
					end
				else
					tests_passed += 1
					print(` {if tests_passed < 10 then " " else ""}{tests_passed} {colors.yellow("test successful")}: {path}`)
					table.insert(results, {
						path = path,
						ok = true,
						err = "",
					})
					local ignore_result = check_ignored(path)
					if ignore_result then
						for _, ignored_function in ignore_result do
							print(`    ⚠️  test case '{ignored_function.name}' ignored due to reason: {ignored_function.reason}`)
						end
					end
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

	print(`{colors.yellow("passed")}: {tests_passed} ({assert_count} total asserts!)`)
	print(`{colors.red("failed")}: {tests_failed}`)

	local current_fail_index = 0

	for _, result in results do
		if result.ok then
			continue
		end
		current_fail_index += 1
		print(`{current_fail_index}    {result.path}: {result.err}`)
		-- we also want to write to stderr so that failing a test fails CI and Build Action
		output.ewrite(`{current_fail_index}    {result.path}: {result.err}\n`)
	end
end

-- local t0 = os.clock()
-- serial()
-- print(os.clock() - t0) -- 45 seconds

local t1 = os.clock()
parallel() -- usually 14 to 14.9 seconds
print(`seconds: {colors.bold.blue(tostring(os.clock() - t1))}`)

local ignored_tests_by_file: { [string]: { IgnoredFunction } } = {} do
	for function_name, info in ignored_functions do
		if not ignored_tests_by_file[info.test_path] then
			ignored_tests_by_file[info.test_path] = {}
		end
		local ignored_for_current_file = ignored_tests_by_file[info.test_path]
		table.insert(ignored_for_current_file, {
			name = function_name,
			reason = info.reason,
		})
	end
end

for file, ignored in ignored_tests_by_file do
	print(`\n ⚠️  ignored cases in '{file}':`)
	for i, fn in ignored do
		print(`  {i}. {fn.name}: {fn.reason}`)
	end
end

for file, reason in invalid_test_files do
	print(`\n{colors.bold.red("[INVALID TEST FILE]:")} '{file}' ({reason})`)
end
