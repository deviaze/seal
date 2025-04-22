local fs = require("@std/fs")

local module = {}

function module.execute()
	error("hi")
	-- print(debug.traceback())
end

function module.readimpossiblefile()
	return fs.readfile("./hmm")
end

return module