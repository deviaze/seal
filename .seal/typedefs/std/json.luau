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
	--- encodes a table as json; by default this encodes as a pretty-formatted string; use `json.raw` for a condensed version instead
	encode: (t: JsonData, options: EncodeOptions?) -> string,
	--- encodes a table as json in a condensed fashion for passing as data (without newlines, not as easily readable)
	raw: (t: JsonData) -> string,
	decode: (json: string) -> JsonData,
	readfile: (path: string) -> JsonData,
	writefile: (path: string, data: JsonData, options: EncodeOptions?) -> (),
	writefile_raw: (path: string, data: JsonData) -> (),
	--- returns an object that serializes to json's `null`
	null: () -> any,
	--- Treat `t` as an array (will always encode as `[]` even when empty).
	---
	--- If `t` isn't provided, returns a new array-like table that will serialize to a json array.
	array: <T>(t: { T }?) -> { T }
}

return {} :: json