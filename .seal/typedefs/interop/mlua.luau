--[=[
Interop with seal's underlying mlua layer
]=]
local mlua = {}

--- Returns `true` if `n` is an mlua Integer, or `false` if it's an `mlua` Number
---
--- Note, this is important (and therefore annoying) because large integer numbers, 
--- such as `1231231231234445` are internally represented as mlua `LuaNumber`s and not mlua `LuaInteger`s; 
--- this breaks Rust-side functions that expect a `LuaInteger` and don't properly handle the `LuaNumber` case.
@checked
function mlua.isint(n: number): boolean
	return nil :: any
end

--- Returns `true` if `e` is an mlua Error, false otherwise
@checked
function mlua.iserror(e: any): boolean
	return nil :: any
end

return mlua