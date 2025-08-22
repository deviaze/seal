--!nolint LocalShadow
local _, cwd = pcall(function()
	return require("@std/env").cwd() -- pcalling here to prevent downstream panic in err.rs if cwd doesn't exist
end)

if typeof(cwd) == "error" then
	warn("Your current working directory might've been removed! Try reloading your editor.")
	cwd = "nonexisting/removed directory"
end

local function parse_traceback(traceback: string): string
	local new_traceback = traceback
		:gsub("%[string ", "[")
		:gsub("%s*%[C%]: in %?", "")
		:gsub(": in %?", ": in top level")
		:gsub(cwd :: string, ".")

	return new_traceback
end

return parse_traceback