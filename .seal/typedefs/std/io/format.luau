--[=[
formats output in the same colorful way as `print()`
### Usage:
```luau
local format = require("@std/io/format")
local net = require("@std/net")

local result = net.get {
	url = "https://somewhere.endpoint/json",
	params = {
		user = "someusername"
	}
}

if result.ok then
	print(`somewhere says: {format(result:decode())}`)
end
````
]=]
local function format(object: any): string
	return nil :: any
end

return format