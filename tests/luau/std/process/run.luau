local process = require("@std/process")
local env = require("@std/env")

local seal_path = env.executable_path

if env.os ~= "Windows" then
	local res = process.run {
		program = "cd ./src; pwd",
		shell = "sh",
	}
	assert(res.ok == true, `result failed: {res.stderr}`)
end

local function nonexistentprogram()
	local s, err = pcall(function()
		return process.run({
			program = "idontexist",
		})
	end)
	assert(tostring(err):match("unable to run the program"), "why do you have a program named 'idontexist'???")
end

nonexistentprogram()

local failed_res = process.run {
	program = seal_path,
	args = { "eval", "process.exit(1)"}
}

local function attempttounwrapfailedrunresult()
	local success, result = pcall(function()
		return failed_res:unwrap()
	end)
	assert(tostring(result):match("Attempt to"), "RunResult:unwrap() broke?")
end

attempttounwrapfailedrunresult()

local function simpledefaults()
	local string_default = failed_res:unwrap_or("hi")
	assert(string_default == "hi", "string default didnt work")
end

simpledefaults()

local function computedresult()
	local uh_oh_src = [[
		local output = require("@std/io/output")
		output.ewrite(`uh oh {math.random(1, 99)}`)
		process.exit(1)
	]]
	local fails_and_writes_to_stderr_default = process.run {
		program = seal_path,
		args = { "eval", uh_oh_src:gsub("\n", "") }
	}:unwrap_or(function(result: process.RunResult)
		local stderr = result.stderr
		return `it's only a paper {result.stderr:match("(%d+)")}`
	end)
	assert(fails_and_writes_to_stderr_default:match("it's only a paper %d%d?"), "default computed doesn't match")
end

computedresult()