local mlua = require("@interop/mlua")

local i = 1200
local f = 1200.01
local bignumber = 1231231231234445

assert(mlua.isint(i) == true, "int is int")
assert(mlua.isint(f) == false, "number is false")
assert(mlua.isint(bignumber) == true, "big integer number actually represented as LuaInteger not LuaNumber (mlua 0.10.3+)")

local success, result = pcall(function()
	return mlua.isint("hello world" :: any)
end)

assert(success == false, "interop.mlua.isint isnt erroring on invalid input?")

assert(mlua.iserror(result) == true, "mlua error is made of error")
