--!nonstrict
local process = require("@std/process")
local env = require("@std/env")

if env.os ~= "Windows" then
	local res = process.run {
		program = "cd ./src; pwd",
	}
	
	assert(res.ok == true, `result failed: {res.stderr}`)
end

local fail_res = process.run({
	program = "idontexist",
})
assert(fail_res.ok == false, "why do you have a program named `idontexist` on your system?")

local success, result = pcall(function()
	fail_res:unwrap()
end)
assert(tostring(result):match("without a default value!"), "RunResult:unwrap() broke?")
