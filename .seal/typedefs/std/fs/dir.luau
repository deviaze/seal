local common_types = require("./_types")

export type DirectoryTree = common_types.DirectoryTree
export type DirectoryEntry = common_types.DirectoryEntry
export type DirectoryBuilder = common_types.DirectoryBuilder

export type DirLib = setmetatable<{
	--- Creates a `DirectoryEntry` from the directory at `path`, erroring if the directory is NotFound/PermissionDenied, etc.
	from: (path: string) -> DirectoryEntry,
	--- Returns a `DirectoryBuilder` table for `fs.readtree`, `fs.writetree`, etc.
	build: (name: string, tree: DirectoryTree) -> DirectoryBuilder,
	--- Creates a *new* directory at `path`, erroring if an entry already exists there.
	create: (path: string) -> DirectoryEntry,

	--[=[
	Ensures that a directory exists at `path` by trying to create it, catching any AlreadyExists error, and returning a `DirectoryEntry` at that path.

	Similar to `fs.makedir(path, { error_if_exists = false }); fs.dir.from(path)`

	## Usage
	```luau
	-- doesn't replace .vscode if it already exists, but creates it if it doesn't
	local dot_vscode = fs.dir.ensure(".vscode")
	local settings_json = dot_vscode:find("settings.json"):try_file()
	```
	]=]
	ensure: (path: string, create_missing: boolean?) -> DirectoryEntry,

	-- fs.dir.try_remove(path: string) -> (boolean, "Ok" | "PermissionDenied" | "NotFound" | "NotADirectory" | "Other", string?)
	--[=[
		Try to remove directory at `path` using Rust's `fs::remove_dir_all` without erroring in common cases.

		If this function partially fails (removes some but not all subtrees/files in `path`), `try_remove` will return false
		with result "Other", as well as an error kind string that describes what went wrong.

		## Errors
		- if provided invalid arguments (`path` is not a valid utf-8 encoded string that could exist on the filesystem)
	]=]
	try_remove: (path: string) -> (boolean, "Ok" | "PermissionDenied" | "NotFound" | "NotADirectory" | "Other", string?),
	--[=[
		Returns a `DirectoryEntry` corresponding to the user's home directory, erroring if not found.

		## Usage

		```luau
		local zip_downloads = fs.dir.home()
			:expect_dir("Downloads")
			:list(false, function(path: string)
				return str.endswith(path, ".zip")
			end)
		```
	]=] 
	home: () -> DirectoryEntry,
	--[=[
		Constructs a `DirectoryEntry` from the user's current working directory (cwd)
	
		If you're looking for project-relative pathing, I recommend using `fs.dir.project()` instead, as those will work no matter
		where the user is when they execute your code.
	]=]
	cwd: () -> DirectoryEntry,
	--[=[
		Constructs a `DirectoryEntry` from the script's current *seal* project.

		Use this for most project-relative paths instead of `fs.path.cwd` or `fs.dir.cwd` usages.

		## Errors

		- if a *seal* project couldn't be found exactly `n` project parents relative to `script:path()`.
		- use `fs.path.project` instead if you want to get the *seal* project without erroring.

		## Usage

		```luau
		local fs = require("@std/fs")
		local str = require("@std/str")
		
		local input_files = fs.dir.project()
			:expect_dir("input")
			:list(false, function(path: string) 
				return str.endswith(path, ".csv")
			end)
	]=]
	project: (n: number?) -> DirectoryEntry,
}, {
	--[=[
	Convenient and slightly more efficient alternative to `fs.find(path):try_dir()`

	## Usage
	```luau
	local src_dir = fs.dir("./src")
	if src_dir then
		local main_luau = src_dir:expect_file("main.luau")
		main_luau:append('print("meow")')
	end
	```
	]=]
	__call: (self: any, path: string) -> DirectoryEntry?,
}>

return {} :: DirLib