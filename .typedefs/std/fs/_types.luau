-- common types that we don't want cyclic require errors on
local datetime = require("@std/time/datetime")

export type TreeBuilder = {
	--- The `DirectoryTree` being constructed by the `TreeBuilder`.
    inner: DirectoryTree,
	--- Add a file to the DirectoryTree by `name` with `content`
    with_file: (self: TreeBuilder, name: string, content: string) -> TreeBuilder,
	--[=[
		Add a new tree to the DirectoryTree; the second argument should be another `TreeBuilder` from `fs.tree()`

		## Usage
		```luau
		local dir = fs.tree()
		    :with_tree("subtree", fs.tree()
				:with_file("hi.json", '{"hi"}: true')
			)
		```
	]=]
    with_tree: (self: TreeBuilder, name: string, builder: TreeBuilder) -> TreeBuilder,
}

export type FileEntry = {
	--- The name of the file; also called basename, filename, etc. Can also be obtained by calling `fs.path.child` on a path.
	name: string,
	--- A filesystem path to the file; if the `FileEntry` was requested with an absolute path, then this path will be absolute, otherwise it'll be a relative path.
	path: string,
	type: "File",
	-- FileEntry:read() -> string
	--[=[
		Reads the file to string without performing utf-8 validation on the file's contents.

		## Errors
		- if `path` is not valid utf-8
		- the file doesn't exist or is actually a directory
		- you don't have permission to access `path`

		Use `fs.file.try_read` instead if you want to handle`NotFound` and/or `PermissionDenied` explicitly without erroring.

		This function blocks the current Luau VM. To use it in parallel, call it within a child thread from `@std/thread`.
	]=]
	read: (self: FileEntry) -> string,
	--- Returns the file's size (length) in bytes.
	size: (self: FileEntry) -> number,
	-- FileEntry:readlines()
	--[=[
		Iterate over the lines of the file without reading the whole file into memory.

		## Usage
		```luau
		local csv = fs.file("./mybigdata.csv")
		if csv then
			for line_number, line in csv:readlines() do
				print(line)
			end
		end
		```

		## Errors
		- if the file's contents are not valid utf-8
		- user cannot access the file
	]=]
	readlines: (self: FileEntry) -> () -> (number, string),
	-- FileEntry:readbytes(file_offset: number?, count: number?, target_buffer: buffer?, buffer_offset: number?) -> buffer
	--[=[
		Reads the file into a buffer.

		## Parameters
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
	readbytes: (self: FileEntry, file_offset: number?, count: number?, target_buffer: buffer?, buffer_offset: number?) -> buffer,
	-- FileEntry:append(content: string | buffer) -> ()
	--[=[
		Opens the file in append mode and appends `content` to the file.

		Like `fs.writefile`, `content` does not have to be a valid utf-8 encoded string (though please just use a buffer instead)

		## Errors
		- if the file cannot be opened in append mode
		- user does not have permission to append to the file
		- unexpected error writing content to file
	]=]
	append: (self: FileEntry, content: buffer | string) -> (),
	is_valid_utf8: (self: FileEntry) -> boolean,
	-- FileEntry:metadata() -> FsMetadata
	--[=[
		Returns a `FsMetadata` table containing timestamps for creation, modified, and access times, as well as permissions (depends on your operating system)
	]=]
	metadata: (self: FileEntry) -> FsMetadata,
	copy_to: (self: FileEntry, to: string) -> (),
	move_to: (self: FileEntry, to: string) -> (),
	rename: (self: FileEntry, name: string) -> (),
	--- Removes the file at `FileEntry.path`.
	remove: (self: FileEntry) -> (),
}

export type DirectoryEntry = {
	--- The name of the directory; also called basename, etc. Can also be obtained by calling `fs.path.child` on a path.
	name: string,
	--- A filesystem path to the directory; if the `DirectoryEntry` was requested with an absolute path, then this path will be absolute, 
	--- otherwise it'll be a relative path.
	path: string,
	type: "Directory",
	--- Returns an an array of basenames of the directory's entries; pass `true` as the second argument to list all files recursively.
	list: (self: DirectoryEntry, recursive: boolean?) -> { string },
	find: (self: DirectoryEntry, name: string, options: { follow_symlinks: boolean?, error_if_permission_denied: boolean? }?) -> FindResult,
	entries: (self: DirectoryEntry) -> { [string]: Entry },
	--- Expect that the directory contains file `name`, returning its `FileEntry` or otherwise error.
	expect_file: (self: DirectoryEntry, name: string) -> FileEntry,
	--- Expect that the directory contains directory `name`, returning its `DirectoryEntry` or otherwise error.
	expect_dir: (self: DirectoryEntry, name: string) -> DirectoryEntry,
	-- DirectoryEntry:add_file(name: string, content: string | buffer) -> DirectoryEntry
	--[=[
		Add or overwrite the file named `name` with `content`, returning the original `DirectoryEntry` for chaining.

		## Usage
		```luau
		local src = fs.dir.ensure("./src")
		    :add_file("main.luau", 'print("hi")')
			:add_file("byte.luau", 'print("munch")')
		```

		## Errors
		- if name is not a valid, utf-8 encoded filename
		- user does not have permission to write files in the directory
	]=]
	add_file: (self: DirectoryEntry, name: string, content: string | buffer) -> DirectoryEntry,
	-- DirectoryEntry:add_tree(name: string, builder: TreeBuilder) -> DirectoryEntry
	--[=[
		Add a new directory tree to the directory from a `TreeBuilder`, returning the ***original*** `DirectoryEntry` for chaining.

		## Usage
		```luau
		local src = fs.dir.ensure("./src")
			:add_tree("libraries", fs.tree()
				:with_file("Lists.luau", lists_src)
			)
		```

		## Errors
		- if `name` is not a valid utf-8 encoded directory name
		- the directory already contains a directory named `name` (or it's a file)
	]=]
	add_tree: (self: DirectoryEntry, name: string, builder: TreeBuilder) -> DirectoryEntry,
	-- DirectoryEntry:metadata() -> FsMetadata
	--[=[
		Returns a `FsMetadata` table containing timestamps for creation, modified, and access times, as well as permissions (depends on your operating system)
	]=]
	metadata: (self: DirectoryEntry) -> FsMetadata,
	copy_to: (self: DirectoryEntry, to: string) -> (),
	move_to: (self: DirectoryEntry, to: string) -> (),
	rename: (self: DirectoryEntry, name: string) -> (),
	--- Removes the directory at `DirectoryEntry.path`, alongside all its contents.
	remove: (self: DirectoryEntry) -> (),
}

export type Entry = FileEntry | DirectoryEntry

export type FsMetadata = {
	--- A UTC DateTime representing when the `Entry` was created.
	--- This field is optional because it might not be available on all platforms.
	created_at: datetime.DateTimeUtc?,
	--- A UTC DateTime representing when the `Entry` was last modified.
	--- This field is optional because it might not be available on all platforms.
	modified_at: datetime.DateTimeUtc?,
	--- A UTC DateTime representing when the `Entry` was last accessed.
	--- This field is optional because it might not be available on all platforms.
	accessed_at: datetime.DateTimeUtc?,
	permissions: {
		--- Whether the `Entry` is read-only or not. Should be accessible on both Windows and Unix-like operating systems.
		readonly: boolean,
		--- Represents the numeric Unix permission bits for the `Entry`, combining read, write, and execute permissions
		--- for owner, group, and others. This field is optional because it's not available on Windows.
		unix_mode: number?,
	}
}

export type FindResult = { -- can't make this type more accurate w/ unions bc it breaks (new) typesolver
	--- `true` if the find operation succeeded ("File" | "Directory" | "Symlink"), otherwise `false` ("NotFound" | "PermissionDenied")
	ok: boolean,
	path: string,
	type: "File" | "Directory" | "Symlink" | "NotFound" | "PermissionDenied",
	--[=[
		Checks if `FindResult.path` exists on the filesystem.

		## Usage
		```luau
		if current_dir:find(".luaurc"):exists() then
			luaurc_path = path.join(current_dir.path, ".luaurc")
		end
		```

		## Errors
		- if permission denied
	]=]
	exists: (self: FindResult) -> boolean,
	--[=[
		Attempt to create a `FileEntry` from the `FindResult`, returning it or `nil` if unsuccessful.

		## Usage
		```luau
		local main_luau = fs.find("./src/main.luau"):try_file()
		if main_luau then
			process.shell(`seal {main_luau.path}`):unwrap()
		end
		```
	]=]
	try_file: (self: FindResult) -> FileEntry?,
		--[=[
		Attempt to create a `DirectoryEntry` from the `FindResult`, returning it or `nil` if unsuccessful.

		## Usage
		```luau
		local src = fs.find("./src"):try_dir()
		if src then
			print(src:list(--[[recursive = ]] true))
		end
		```
	]=]
	try_dir: (self: FindResult) -> DirectoryEntry?,
	--- Create a `FileEntry` from the `FindResult`, erroring if the file doesn't exist.
	unwrap_file: (self: FindResult) -> FileEntry,
	--- Create a `DirectoryEntry` from the `FindResult`, erroring if the directory doesn't exist.
	unwrap_dir: (self: FindResult) -> DirectoryEntry,
}

export type FileBuilder = {
	name: string,
	type: "File",
	content: string,
}

export type DirectoryBuilder = {
	name: string,
	type: "Directory",
	children: DirectoryTree,
}

export type DirectoryTree = {
	FileBuilder | DirectoryBuilder
}

return {}