--[=[
Of course *you'd* require the whole standard library at once...
]=]
local std = {
	fs = require("@std/fs"),
	str = require("@std/str"),
	env = require("@std/env"),
	io = require("@std/io"),
	colors = require("@std/colors"),
	format = require("@std/io/format"),
	time = require("@std/time"),
	datetime = require("@std/time/datetime"),
	process = require("@std/process"),
	serde = require("@std/serde"),
	json = require("@std/json"),
	net = require("@std/net"),
	thread = require("@std/thread")
}

return std