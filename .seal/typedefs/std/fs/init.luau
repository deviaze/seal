
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

	--> fs.watch(paths: string | { string }, options: WatchOptions)
	--[=[
		Watch for filesystem changes on one or more `paths`.

		- `WatchOptions.recursive`: defaults `true`; may produce duplicate events if any `paths` overlap,
		- `WatchOptions.timeout_ms`: in milliseconds, after which the `fs.watch` iterator will return a `"None"` event instead of blocking the VM.
		Defaults to 10 milliseconds.

		Note that filesystem watching is inherently *messy* and *platform specific*!
		
		In many cases, instead of just relying on the exact events this api provides to know which
		files were created or removed, it may be better just to check the filesystem yourself 
		with `fs.listdir/entries`, and use `fs.watch` as a trigger for knowing when to check.

		Because you can get duplicated events, it's better to add your own debounces and if you're relying on counts,
		
		Don't expect that you'll get the same events every time, or that the same operation will map to the same specific `WatchKind`
		on all platforms (though they *should* be similar.)

		## Usage

		- You probably want to ignore `"Access"` since it's noisy (unless you need to know which files are currently open)
		- If you want to listen for files getting added/modified, don't rely on certain kinds/categories.
			- Check for `WatchEventInfo.is_write` instead of relying on `Modify::Data` or `Create` or `Close::Write`.
		- If you don't want to loop forever, `break` if when you keep encountering `category == "None"` for a while.
		- To make this function nonblocking (as best as possible), pass 0 milliseconds to `timeout_ms`. 
			- You'll probably still want to loop it/put it in a function and watch out for `"None"` events.

		## Examples

		Run a Lune script whenever a file in ./src/** changes:
		```luau
		local fs = require("@std/fs")
		local str = require("@std/str")
		local process = require("@std/process")

		local serializer_script = fs.path.join(".", ".lune", "instance_serializer.luau")

		for category, event in fs.watch("./src") do
			if category == "Access" or category == "None" then
				continue -- ignore these, we only need "None" if we want to break loop
			elseif event.is_write then
				local modified_path = event.paths[1]
				local result = process.run {
					program = "lune",
					args = {"run", serializer_script, modified_path}
				}
			end
		end
		```

		Watch only .json config files for changes:

		```luau
		local options: fs.WatchOptions = {
			recursive = false,
			timeout_ms = 2, -- blocks vm for max of 2 milliseconds
		}
		
		local files = fs.listdir(
			"./src", , event i
			true, -- recursive
			function(path: string) -- filter
				return if string.match(path, "%.json$") or string.match(path, "%.luaurc") 
					then true 
					else false
			end
		)
		
		for _, event in fs.watch(files, options) do
			if event.is_write then
				local modified = fs.path.child(event.paths[1])
				if modified then
					print(`Config file modified: {modified}!`)
				end
			end
		end
		```
		
		Manually poll a few times:

		```luau
		local options = {
			recursive = true,
			timeout_ms = 2, -- check for 2 seconds
		}
		time.wait(1)
		local poll = fs.watch("./src", options)
		local cat, event = poll()
		if cat == "None" then
			time.wait(1) -- retry
			cat, event = poll()
		end
		if cat == "None" then
			print("not found")
		end
		```

		With a custom timeout:

		```luau
		local start_time = os.clock()
		for category, event in fs.watch(script:parent()) do
			if category == "None" and os.clock() - start_time > 5 then
				break
			end
			if event.is_write then
				print(event)
			end
		end
		print("hi after 5 seconds")
		```

		This function uses the Rust `notify` crate as its backend; please refer to its documentation for more specifics.
	]=]
	watch: (paths: string | { string }, options: WatchOptions?) -> () -> (WatchEventCategory, WatchEventInfo),

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

export type WatchOptions = {
	recursive: boolean?,
	timeout_ms: number?,
}

--[=[
	Top level categories to filter events by.

	Some usage notes:
	- You should probably ignore `"Access"` unless you need to know which files are currently open.
	- Don't rely on solely `"Create"` or `"Modify::Data"` to check if a file was created/modified, use `WatchEventInfo.is_write` instead.
	- `"None"` indicates a timeout was reached; use it to early exit or `break` without blocking the VM.
]=]
export type WatchEventCategory = 
	| "Read" -- note that Read ~= open for reading (which is in Open)
	| "Execute"
	| "Open"
	| "Close"
	| "Access"
	| "Create"
	| "Rename"
	| "Modify::Data"
	| "Modify::Metadata"
	| "Modify::Other"
	| "Remove"
	| "Other"
	| "Unknown"
	| "None"

export type WatchEventInfo = {
	paths: { string },
	kind: WatchKind,
	--- if the event is *most likely* a write event (`Create::File` or `Modify::Data` or `Close::Write`)
	is_write: boolean,
}

--- Represents the specific Event.WatchKind from notify.
---
--- Note that relying on these is inherently unreliable as notify tends to combine related events.
--- Especially if they're received within a short interval of each other.
--- 
--- `None::Timeout` is fired if no events have been seen when `WatchOptions.timeout_ms` elapses
--- for an iteration of `fs.watch`. This allows you to break early without indefinitely blocking the Luau VM.
export type WatchKind =
    -- Read -- AccessKind::Read
    | "Read"

    -- Execute -- AccessKind::Open(Execute)
    | "Open::Execute"

    -- Open -- AccessKind::Open(_)
    | "Open::Read"
    | "Open::Write"
    | "Open::Other"

    -- Close -- AccessKind::Close(_)
    | "Close::Execute"
    | "Close::Read"
    | "Close::Write"
    | "Close::Other"
    | "Close::Any"

    -- Access -- AccessKind::Any, AccessKind::Open(Any), AccessKind::Other
    | "Open::Any"
    | "Access::Any"
    | "Access::Other"

    -- Create -- CreateKind
    | "Create::File"
    | "Create::Directory"
    | "Create::Other"
    | "Create::Any"

    -- Rename -- ModifyKind::Name
    | "Rename::Any"
    | "Rename::From"
    | "Rename::To"
    | "Rename::Both"
    | "Rename::Other"

    -- Modify::Data -- ModifyKind::Data
    | "Modify::Data"
    | "Modify::Data::Content"
    | "Modify::Data::Size"
    | "Modify::Data::Other"

    -- Modify::Metadata -- ModifyKind::Metadata
    | "Modify::Metadata::AccessTime"
    | "Modify::Metadata::WriteTime"
    | "Modify::Metadata::Ownership"
    | "Modify::Metadata::Permissions"
    | "Modify::Metadata::Extended"
    | "Modify::Metadata::Other"
    | "Modify::Metadata::Any"

    -- Modify::Other -- ModifyKind::Any, ModifyKind::Other
    | "Modify::Any"
    | "Modify::Other"

    -- Remove -- RemoveKind
    | "Remove::File"
    | "Remove::Directory"
    | "Remove::Other"
    | "Remove::Any"

    -- Other -- EventKind::Other
    | "Other"

    -- Unknown -- EventKind::Any
    | "Unknown"

	-- No event received after `timeout_ms` ms elapsed
	| "None::Timeout"


return {} :: fs