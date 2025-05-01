local env = {}
--[=[
A stdlib to interact with the script's running environment.
]=]
type env = {
	--- a list of arguments passed to the program
	args: {string},
	--- your operating system
	os: "Windows" | "Linux" | "Android" | "MacOS" | "Other",
	--- the path of the executable
	executable_path: string,
	script_path: string,
	--[=[ 
	Get the current working directory of the running process.

	Errors if the `cwd` doesn't exist or otherwise isn't accessible (permission denied).
	]=]
	cwd: () -> string,
	--[=[
	Gets an environment variable in the current process.
	]=]
	getvar: (key: string) -> string?,
	--[=[
	Sets an environment variable in the current process.

	Note, this function is **unsafe** in multithreaded contexts on Linux.
	]=]
	setvar: (key: string, value: string) -> string,
	--[=[
	Removes an environment variable in the current process.

	Note, this function is **unsafe** in multithreaded contexts on Linux.
	]=]
	removevar: (key: string) -> nil,
}

return env :: env