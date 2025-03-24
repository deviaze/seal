export type PathLib = {
	--[=[
		Joins path components together in a cross-platform manner.
	]=]
	join: (...string) -> string,
	exists: (path: string) -> boolean,
	--[=[
		Returns the canonical (absolute) form of `path` using Rust's `std::fs::canonicalize`, resolving symlinks and intermediate components.

		Errors if the requested path doesn't exist on the filesystem or is invalid.
	]=]
	canonicalize: (path: string) -> string,
	--[=[
		Returns the absolute path of `path` without checking the filesystem.

		Use this function if your path may or may not exist (yet).
	]=]
	absolutize: (path: string) -> string,
	--[=[
		Returns the path of the parent directory `n` (default = 1) parents to the left of `path`

		## Usage
		```luau
		local fs = require("@std/fs")
		local env = require("@std/env")
		local path = fs.path

		local cwd = env.current_working_directory
		local parent_dir = path.parent(cwd)		
		```
	]=]
	parent: (path: string, n: number?) -> string?,
	--- the farthest child/leaf/node of the path, ex. `path.child("./src/main.luau") == "main.luau"`
	child: (path: string) -> string?,
	--- returns the user's home directory
	home: () -> string,
}

return {} :: PathLib