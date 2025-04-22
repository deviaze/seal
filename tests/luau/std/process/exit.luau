-- requiring a file that calls process.exit should not panic (Lune's behavior)

local fs = require("@std/fs")
local env = require("@std/env")
local process = require("@std/process")

local test_dir = fs.dir.from(script:parent())

local file_immediately_exits_src = [[
	local process = require("@std/process")
	process.exit(0)
	print("meow")
]]

local main_test_file_src = [[
	local env = require("@std/env")
	local process = require("@std/process")

	local file_to_require = env.args[1]
	local otherfile = (require :: any)(`./tests/{file_to_require}`)
	if file_to_require == "call_exit_function" then
		otherfile.exit(0)
	end
	print("meow")
]]

local call_exit_function_src = [[
	local process = require("@std/process")
	local time = require("@std/time")

	local module = {}
	function module.exit(code: number?): never
		time.wait(0.25) -- wait a little
		process.exit(code)
		print("meow")
	end
	return module
]]

local file_exits_with_not_ok_src = [[
	local process = require("@std/process")
	return process.exit(1)
]]

-- remove exit-tests if alr exists
fs.dir.try_remove(test_dir:join("exit-tests"))

local exit_tests = test_dir:add_tree("exit-tests", fs.tree()
	:with_file("exit_main.luau", main_test_file_src)
	:with_tree("tests", fs.tree()
		:with_file("immediately_exits.luau", file_immediately_exits_src)
		:with_file("call_exit_function.luau", call_exit_function_src)
		:with_file("not_ok.luau", file_exits_with_not_ok_src)
	)
):expect_dir("exit-tests")

local test_main_luau = exit_tests:expect_file("exit_main.luau")

local immediately_result = process.run {
	program = env.executable_path, -- usually ./target/debug/seal
	args = { 
		test_main_luau.path,
		"immediately_exits"
	}
}

local called_result = process.run { 
	program = env.executable_path,
	args = {
		test_main_luau.path,
		"call_exit_function"
	}
}

local not_ok_result = process.run {
	program = env.executable_path,
	args = {
		test_main_luau.path,
		"not_ok"
	}
}

assert(immediately_result.ok == true, "requiring file that exits should work")
assert(called_result.ok == true, "calling function that exists should work")
assert(not_ok_result.ok == false, "calling process.exit with exit code 1 should result in not ok result")

assert(not immediately_result.stdout:match("meow"), "meow should never be printed")
assert(not called_result.stdout:match("meow"), "meow should never be printed even when called from function that waits")
assert(not not_ok_result.stdout:match("meow"), "meow should never be printed even when exit code 1")

exit_tests:remove()