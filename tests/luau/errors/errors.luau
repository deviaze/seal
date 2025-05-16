-- TODO: properly test errors without making tests too brittle
-- local mlua = require("@interop/mlua")
local module = require("./another_module")

local function main()
	script.context = "rfing"
	module.readimpossiblefile()
	return module.execute()
end

local success, result = pcall(main :: any)
-- assert(mlua.iserror(result), "error should be made of error")
