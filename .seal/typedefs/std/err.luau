--- Simple library for managing `error` types from seal.
export type err = {
	--[=[
		Create an `error` with a custom error message. This allows you to return result-like unions that can be `typeof` checked.

		## Usage
		```luau
		local err = require("@std/err")

		local function canfail(): string | error
			if not somecheck() then
				return err.message("whoops we failed")
			end
			return "success"
		end

		local res = canfail()
		if typeof(res) == "error" then
			-- error handling
		else
			-- res should be narrowed to `string`
		end
		```
	]=]
	message: (string) -> error,
	--[=[
		Wraps an error message with the stack traceback at the location this function was called.

		Unlike `err.message`, errors created with this function are red colored and contain their own error tracebacks.
	]=]
	wrap: (message: string) -> error,
	--- Prettifies an `error` from `pcall` or any of the error returning functions.
	format: (err: error) -> string,
	--- Gets and formats the stack traceback at the current location.
	traceback: () -> string,
	--- Takes any error and throws it (causing an error). 
	---
	--- If the error already has stack traceback information, throwing the error causes two stack tracebacks to appear (one from the error itself, and a new one because we're causing an error).
	throw: (err: error) -> never,
}

export type Result<T> = T | error

return {} :: err