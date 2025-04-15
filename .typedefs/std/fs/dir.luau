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