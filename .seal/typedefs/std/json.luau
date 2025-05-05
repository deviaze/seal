type EncodeOptions = {
	--- with tabs/newlines/spaces for easy reading (default true)
	pretty: boolean?,
	--- sorted in alphanumeric order (default false)
	sorted: boolean?,
}

export type JsonData = { [any]: any }

--[=[
Easily manipulate JSON (JavaScript Object Notation) data.

## Usage
```luau
local json = require("@std/json")
local data = json.readfile("./animals.json") :: { cats: number, dogs: number }
data.cats += 1
json.writefile("./animals.json", data)
```
]=]
type json = {
	--- encodes json with newlines, readable
	encode: (t: JsonData, options: EncodeOptions?) -> string,
	--- encodes json data without newlines, etc.
	encode_raw: (t: JsonData) -> string,
	decode: (json: string) -> any,
	readfile: (path: string) -> JsonData,
	writefile: (path: string, data: JsonData, options: EncodeOptions?) -> (),
	writefile_raw: (path: string, data: JsonData) -> ()
}

return {} :: json