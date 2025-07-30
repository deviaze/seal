
local dirlib = require("@self/dir")
local pathlib = require("@self/path")
local filelib = require("@self/file")
local common_types = require("@self/_types")

export type fs = {
	-- fs.readfile(path: string) -> string
	--[=[
		Reads the file at `path` to string, without performing utf-8 validation on the file's contents.

		## Errors
		- if `path` is not valid utf-8
		- the file doesn't exist or is actually a directory
		- you don't have permission to access `path`

		Use `fs.file.try_read` instead if you want to handle`NotFound` and/or `PermissionDenied` explicitly without erroring.

		This function blocks the current Luau VM. To use it in parallel, call it within a child thread from `@std/thread`.
	]=]
	readfile: (path: string) -> string,

	-- fs.readbytes
	--[=[
		Reads the file at `path` into a buffer.

		This function has 3 common variants:
		- Read the whole file into a new buffer: `fs.readbytes(path: string)`
		- Partially read a file into a new buffer: `fs.readbytes(path: string, file_offset: number, count: number)`
		- Partially read a file into an existing buffer: `fs.readbytes(path: string, file_offset: number, count: number, target_buffer: buffer, buffer_offset: number?)

		## Parameters
		- `path`: must be a valid, utf-8 encoded string representing an accessible file's path on your filesystem
		- `file_offset`: the number of bytes from the start of the file to the start of the portion you want to read. Default is `0` (start of file)
		- `count`: the number of bytes you want to read (starting from `file_offset`) and copy into the buffer
		- `target_buffer`: an optional buffer you want to write into; if not specified, a new buffer will be created for you
		- `buffer_offset`: an optional number of bytes from the start of the `target_buffer` provided; this is useful if you're filling the same buffer from multiple calls

		## Returns
		- `target_buffer`: the exact same `target_buffer` provided, or a new buffer if not provided

		## Errors
		- if `path` is not a file, not valid utf-8, is actually a directory, not found or permission denied, etc.
		- `file_offset`, `count`, or `buffer_offset` cannot be converted into positive mlua Integers
		- provided `target_buffer` is too small (`buffer_offset` + `count` > buffer size)
		- attempt to read a nonexistent portion of file (`file_offset` + `count` > file size)

		This function blocks the current Luau VM. To use it in parallel, call it within a child thread from `@std/thread`.
	]=]
	readbytes: (path: string, file_offset: number?, count: number?, target_buffer: buffer?, buffer_offset: number?) -> buffer,

	-- fs.readlines(path: string) -> () -> (number, string)
	--[=[
		Iterate over the lines of a file without reading the whole file into memory.

		This function returns a normal iterator function, so if you save the return of `fs.readlines` to a variable, you can keep calling it for the next line!

		## Errors
		- if `path` is not valid utf-8, doesn't point to a file, not found or permission denied, etc.

		## Usage
		```luau
		for line_number, line in fs.readlines("./myfile.txt") do
			print(`{line_number} says {line}`)
		end

		local nextline = fs.readlines("./myfile.txt")
		local _, line1 = nextline()
		local _, line2 = nextline()
		```
	]=]
	readlines: (path: string) -> () -> (number, string),

	-- fs.writefile(path: string, content: string | buffer) -> ()
	--[=[
		Writes `content` to the file at `path`, overwriting any file that already exists there.

		Note that `content` may be either a string or a buffer; in either case, `content` does not need to be utf-8 encoded.

		## Usage
		```luau
		local content = getcontent()
		fs.writefile("./myfile.txt",content)
		```
		## Errors
		- if `path` is not a valid, utf-8-encoded path to a file or empty location on the filesystem
		- path already exists on the filesystem and is a directory
		- the user does not have permission to access `path`

		This function blocks the current Luau VM. To use it in parallel, call it within a child thread from `@std/thread`.
	]=]
	writefile: (path: string, content: string | buffer) -> (),

	-- fs.removefile(path: string) -> ()
	--[=[
		Removes a regular file at `path` without following symlinks.

		## Usage
		```luau
		fs.removefile("./bad.exe")
		```

		This function blocks the current Luau VM. To use it in parallel, call it within a child thread from `@std/thread`.
	]=]
	removefile: (path: string) -> (),

	-- fs.readtree(path: string) -> DirectoryTree
	--[=[
		Recursively read contents of directory at `path` into a `fs.DirectoryTree` that can be passed into `fs.writetree` and `DirectoryEntry:add_tree` apis.
	]=]
	readtree: (path: string) -> DirectoryTree,

	-- fs.writetree(path: string, tree: TreeBuilder | DirectoryTree) -> ()
	--[=[
		Writes a new directory tree at `path` (which includes the directory's name) from `tree`:

		## Usage
		```luau
		-- using TreeBuilders from fs.tree()
		fs.writetree("./tests", fs.tree()
			:with_file("run.luau", test_runner_src)
			:with_tree("simple-tests", fs.tree()
				:with_file("cats.spec.luau", cats_src)
				:with_file("seals.spec.luau", seals_src)
			)
		)
		-- or using a return from fs.readtree:
		local all_tests = fs.readtree("./all_tests")
		local applicable_tests: fs.DirectoryTree = {} do
			for _, entry in all_tests do
				if entry.type == "File" and string.find(entry.name, "spec%.luau$") then
					table.insert(applicable_tests, entry)
				end
			end
		end
		fs.writetree("./applicable_tests", applicable_tests)
		```

		## Errors
		- if `path` not a valid utf-8 string representing a path on the filesystem
		- an entry already exists at `path` or user does not have permission to access `path`
		- `tree` is not a valid `fs.TreeBuilder` or `fs.DirectoryTree` (`{ fs.FileBuiler | fs.DirectoryBuilder }`) 

		Use fs.makedir instead if you  just want to make an empty directory.

		This function blocks the current Luau VM. To use it in parallel, call it within a child thread from `@std/thread`.
	]=]
	writetree: (path: string, tree: TreeBuilder | DirectoryTree) -> (),

	-- fs.removetree(path: string) -> ()
	--[=[
		Removes a directory tree or an empty directory at `path` by calling Rust's `fs::remove_dir_all`, without following symlinks.
		
		```luau
		local victim_folder = fs.path.join(fs.path.cwd(), "badfolder")
		fs.makedir(victim_folder, { error_if_exists = false })
		fs.removetree(victim_folder)
		```

		Please use this function carefully.

		## Errors
		- if `path` is not a valid utf-8 encoded path to a directory on the filesystem
		- user does not have permission to access `path`
		- `fs.removetree` fails to remove some (or all) files and directories within `path`
	]=]
	removetree: (path: string) -> (),

	-- fs.makedir(path: string, options: { create_missing: boolean?, error_if_exists: boolean? }?) -> boolean
	--[=[
		Create an empty directory at `path` according to (an optional) `options` table.

		By default, `create_missing` is set to `false` and `error_if_exists` is set to `true`.
		
		- Enable `create_missing` to create any missing intermediate directories (such as `"recipes"` in `"./food/recipes/pumpkin_pie.md"`) using Rust's `fs::create_dir_all`.
		- Disable `error_if_exists` if you expect the directory to already exist in normal use and only want to make the directory if it doesn't.

		If you want to ensure that a directory exists (like `fs.makedir(d, { error_if_exists = false })`), and get a `DirectoryEntry`, use `fs.dir.ensure` instead.

		## Usage
		```luau
		fs.makedir(fs.path.join(fs.path.cwd(), "Config", "Editor", "Formatting"), {
			create_missing = true,
			error_if_exists = false,
		})
		```

		# Errors
		- if `path` is not a valid utf-8 encoded path
		- a directory already exists at `path` and `options.error_if_exists` is omitted or set to `true`
		- user does not have permission to access or to create a directory at `path`
		- a file unexpectedly exists at `path`
		- an intermediate component directory of `path` is missing and `create_missing` is omitted or set to `false`
	]=]
	makedir: (path: string, options: { create_missing: boolean?, error_if_exists: boolean? }?) -> boolean,

	-- fs.listdir(path: string, recursive: boolean?, filter: FilterFn?) -> { string }
	--[=[
		Returns an array of all child paths of directory `path`, relative to the passed path.
		
		This means paths from `fs.listdir` can be directly passed into other `fs` library functions.

		Pass `true` as the second parameter to recursively enumerate all files in the directory tree.

		If a filter function is passed, only paths that pass the filter are included.

		## Usage
		```luau
		local test_files: { string } = fs.listdir("./tests", --[[recursive =]] true)

		-- all .luau files
		local luau_files = fs.listdir("./tests", true, function(path: string)
			return if string.match(path, "%.luau$") then true else false
		end)
		```
		
		## Errors
		- if `path` is not a valid, utf-8 encoded string
		- `path` does not exist in the filesystem or is not a directory
		- user does not have permission to access `path`
	]=]
	listdir: (path: string, recursive: boolean?, filter: ((path: string) -> boolean)?) -> { string },

	-- fs.move(from: string, to: string) -> ()
	--[=[
		Move a regular file or directory `from` a path `to` a new path.

		TODO: streamline fs.move and fs.copy with Entry:move_to and Entry:copy_to.

		## Errors 
		- if `from` or `to` are not valid utf-8 encoded paths
		- `from` does not exist on the filesystem
	]=]
	move: (from: string, to: string) -> (),

	-- fs.copy(source: string, destination: string) -> ()
	--[=[
		Copy a regular file or directory from `source` to `destination`.

		TODO: streamline fs.move and fs.copy with Entry:move_to and Entry:copy_to.
	]=]
	copy: (source: string, destination: string) -> (),

	-- fs.find(path: string, options: { follow_symlinks: boolean?, error_if_permission_denied: boolean? }?) -> FindResult
	--[=[
		Check the filesystem for `path`, returning a `fs.FindResult` that's useful for finding `fs.FileEntry` or `fs.DirectoryEntry` to work with.

		This is a multifunctional api, which is usually used to find and unwrap `fs.Entry`-like tables, but is also used for general "finding stuff on the filesystem" usecases.

		Note that `fs.find` and `fs.Entry`-related apis are **not TOCTOU (Time Of Check To Time of Use) safe**; use the `try` apis (`fs.file.try_*` and `fs.dir.try_*`) instead for security or time critical applications.

		## Usage
		Look for a `FileEntry` at `path`:
		```luau
		local file_content: string? = nil
		local file = fs.find("./myfile.txt"):try_file()
		if file then
			file_content = file:read()
		end
		```
		Check if `path` is a file:
		```luau
		if fs.find("./mypath").type == "File" then
			-- code
		end
		```
		Check if we have access to `path`
		```luau
		local result = fs.find(maybeaccesspath, { error_if_permission_denied = false })
		if result.type ~= "PermissionDenied" then
			-- code
		end
		```

		## Errors
		- if `path` is not a valid utf-8 encoded path
		- if user does not have permission to access `path` and options.error_if_permission_denied is unspecified or set `true`
	]=]
	find: (path: string, options: { follow_symlinks: boolean?, error_if_permission_denied: boolean? }?) -> FindResult,

	-- fs.entries(path: string) -> { [string]: Entry }
	--[=[
		Returns a table mapping the paths of the directory at `path` with their `fs.Entry`s.

		## Usage
		```luau
		for path, entry in fs.entries("./src") do
			if entry.type == "File" then
				print(`{entry.name} is a file`)
			elseif entry.type == "Directory" then
				print(`{entry.name} is a directory`)
			end
		end
		```
	]=]
	entries: (path: string) -> { [string]: Entry },

	-- fs.file (lib)
	--[=[
		A sublib for handling operations with files and `fs.FileEntry`s.

		Contains (relatively) TOCTOU-safe apis such as `fs.file.try_read`, etc.

		This library can be called as a function as a convenience alternative for `fs.find(f):try_file()`.
	]=]
	file: filelib.FileLib,

	-- fs.dir (lib)
	--[=[
		A sublib for handling operations with directories and `fs.DirectoryEntry`s.

		This library can be called as a function as a convenience alternative to `fs.find(d):try_dir()`
	]=]
	dir: dirlib.DirLib,

	-- fs.path (lib)
	--[=[
		A sublib for handling file path operations with strings in an ergonomic and cross-platform-compliant manner.

		Commonly used `fs.path` functions include: `fs.path.join` for combining paths and `fs.path.cwd` and `fs.path.home`.
	]=]
	path:  pathlib.PathLib,

	-- fs.tree() -> TreeBuilder
	--[=[
		Returns a `TreeBuilder` for use with `fs.writetree`, `DirectoryEntry:add_tree`, and `TreeBuilder:with_tree` apis.
	]=]
	tree: () -> TreeBuilder,
}

export type DirectoryTree = common_types.DirectoryTree
export type DirectoryBuilder = common_types.DirectoryBuilder
export type TreeBuilder = common_types.TreeBuilder
export type FindResult = common_types.FindResult

export type Entry = common_types.Entry
export type FileEntry = common_types.FileEntry
export type DirectoryEntry = common_types.DirectoryEntry


return {} :: fs