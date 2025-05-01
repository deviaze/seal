local cwd = require("@std/env").cwd()

local function parse_traceback(traceback: string): string
	local new_traceback = traceback
		:gsub("%[string ", "[")
		:gsub("%s*%[C%]: in %?", "")
		:gsub(": in %?", ": in top level")
		:gsub(cwd, ".")

	return new_traceback
end

return parse_traceback