local env = require("@std/env")
local process = require("@std/process")

if env.os == "Linux" or env.os == "Android" then
	local result = process.shell("ls -l")
		:unwrap()
		:split("\n")
	assert(result[1]:match("total ([%d]+)"), "first ls -l result doesnt match")
end

if env.os == "Windows" then
	local res2 = process.shell("Get-ChildItem Env: | Format-Table -AutoSize"):unwrap()
	assert(res2:match("ALLUSERSPROFILE"), "can't find ALLUSERSPROFILE environment variable")
end