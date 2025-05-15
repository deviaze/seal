--[=[
Standard library for formatting and dealing with terminal output.

Usage:
```lua
local output = require("@std/io/output")

-- format a table with colors
local formatted = output.format({
	m = 2,
	d = 3
})

print(formatted)

-- use `p` after it got shadowed for some reason  
local p = getpvalue()
-- oh no i can't use debug print `p` anymore? debug-print to the rescue.
output["debug-print"]({
	p = p,
})
```
]=]
local output = {}

--- prettifies and colorizes any Luau value in the same way as `print`
function output.format(v: any): ...string
	return nil :: any
end

--- reverts output.format or colorizing from print/pp
function output.unformat(v: any): string
	return nil :: any
end

--- simple print and return; equivalent to `p`
function output.sprint<T>(v: T): T
	return nil :: any
end

--- formats any value in simple format style (equivalent to `p`)
function output.sformat(v: any): string
	return nil :: any
end

--- an uncolorized print for debugging that returns the printed values
output["debug-print"] = function(...: any): ...string
	return nil :: any
end

--- like `debug-print` but doesn't print the passed values (just formats them)
output["debug-format"] = function(...: any): ...string
	return nil :: any
end

--- regular colorized `print()` but it returns; same as included global `pp`
output["print-and-return"] = function(...: any): ...string
	return nil :: any
end

--- clears the terminal stdout, aka `cls` or `clear`
function output.clear(): nil
	return nil :: any
end

--- writes contents to stdout
function output.write(contents: string | buffer): nil
	return nil :: any
end

--- writes contents to stderr
function output.ewrite(contents: string | buffer): nil
	return nil :: any
end

return output