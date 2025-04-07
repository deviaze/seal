local fs = require("@std/fs")
local process = require("@std/process")

local test_files = fs.dir.from("./tests/luau/std"):list(true)

local passed_tests = {}
local failed_tests = {}

local ignore_files = {
	"./tests/luau/std/net/server/serve.luau",
	"./tests/luau/std/thread/get-threads/send_request.luau",
	"./tests/luau/std/net/server/client.luau",
	"./tests/luau/std/thread/conc_1.luau",
}

for _, path in test_files do
	if table.find(ignore_files, path) then
		continue
	elseif not path:match("%.luau$") then
		continue
	end
	print(`starting {path}`)

	local result = process.run {
		program = "./target/debug/seal",
		args = { path }
	}
	if result.ok then
		table.insert(passed_tests, path)
	else
		table.insert(failed_tests, {
			file = path,
			err = result.stderr
		})
	end
end

print(`Tests passed: {#passed_tests}\nTests failed: {#failed_tests}`)

for index, test in failed_tests do
	print(`{index}:\n    {test.file}: {test.err}`)
end
