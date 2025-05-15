--[=[
Standard library for handling **terminal** input/output.

You can require the whole `@std/io` lib at once, but it's recommended you require individual components as you need them instead. 

Usage:

```luau
local input = require("@std/io/input")
local output = require("@std/io/output")
local colors = require("@std/io/colors")
local clear = require("@std/io/clear")

local response = input.get("hi, who are you?: ")
if response == "me" then
	print("that's good; me too")
end

-- use output.format to prettify your output!

local some_random_table = {
	a = function() end,
	b = "text",
	c = {
		hey = "i am a human apparently"
	}
}

print(`my very cool table: {output.format(some_random_table)}`)

clear() -- cls/clear
```
]=]
local io = {}

io.input = require("@self/input")
io.colors = require("@self/colors")
io.output = require("@self/output")

return io