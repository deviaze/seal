--[=[
The greatest simplest filesystem standardlib that's ever lived (in luau).

## Usage

```luau
local fs = require("@std/fs")

-- read a file to string
local text = fs.readfile("./somefile.txt")
-- iterate through files in a dir
for path, entry in fs.entries("./somedir") do
	if entry.type == "File" then
		print(entry:read())
	end
end

-- check if a file exists and read it
local f = fs.file("./f.txt")
local content = if f then f:read() else ""

-- Quick reference:
type fs = {
	readfile: (path: string) -> string,
	writefile: (WriteFileOptions) -> (),
	exists: (path: string) -> boolean,
	find: (string | {directory: string } | {file: string}) -> Entry?,
	file: (path: string) -> FileEntry?,
	dir: (path: string) -> DirectoryEntry?,
	entries: (directory: string) -> { [string]: Entry },
	list: (directory: string) -> {string},
	create: (CreateOptions) -> (),
	remove: (RemoveOptions) -> (),
}
export type Entry = DirectoryEntry | FileEntry
export type DirectoryEntry = {
	name: string?,
	type: "Directory",
	path: string,
	list: (self: DirectoryEntry) -> {string},
	entries: (self: DirectoryEntry) -> { [string]: Entry },
	remove: (self: DirectoryEntry) -> (),
}
export type FileEntry = {
	name: string?,
	type: "File",
	path: string,
	extension: string,
	read: (self: FileEntry) -> string,
	remove: (self: FileEntry) -> (),
}
export type WriteFileOptions = {
	path: string,
	content: string,
	overwrite: boolean?,
}
export type RemoveOptions = RemoveFileOptions | RemoveDirectoryOptions
export type RemoveFileOptions = { file: string }
export type RemoveDirectoryOptions = {
	directory: string,
	force: boolean?, --- this defaults to true
	--- only if this is specified and true will you be allowed to remove paths that start in /
	remove_absolute_path: boolean?,
}
```
]=]
local fs = {}

type fs = {
	--- reads a file to string
	readfile: (path: string) -> string,
	--- reads a file to buffer of bytes
	readbytes: (path: string, s: number?, f: number?) -> buffer,
	--- saves/writes a file given WriteFileOptions: {path: string, content: string, overwrite: boolean?}
	writefile: (WriteFileOptions) -> (),
	--- checks if a path exists or not
	exists: (path: string) -> boolean,
	--- finds an Entry if the requested file/directory exists, else returns nil
	find: (string | {directory: string } | {file: string}) -> Entry?,
	--- finds a FileEntry at a given path or nil if the file not found. Equivalent to fs.find { file: string }
	file: (path: string) -> FileEntry?,
	--- finds a DirectoryEntry at a given path or nil if the directory not found. Equivalent to fs.find { directory: string }
	dir: (path: string) -> DirectoryEntry?,
	--- returns a dictionary of Entries by path
	entries: (directory: string) -> {
		[string]: Entry
	},
	list: (directory: string) -> {string},
	--- creates a file, directory, or directory tree with optional content
	create: (CreateOptions) -> (),
	--- removes a file or directory; defaults to force = true
	remove: (RemoveOptions) -> (),
}

export type WriteFileOptions = {
	path: string,
	content: string,
	overwrite: boolean?,
}

export type Entry = (DirectoryEntry | FileEntry)

export type DirectoryEntry = {
	name: string?,
	type: "Directory",
	path: string,
	list: (self: DirectoryEntry) -> {string},
	entries: (self: DirectoryEntry) -> {
		[string]: Entry
	},
	remove: (self: DirectoryEntry) -> (),
	create: (self: DirectoryEntry, options: CreateOptions) -> FileEntry | DirectoryEntry,
	find: (self: DirectoryEntry, options: FindOptions) -> (FileEntry | DirectoryEntry)?
}

export type FileEntry = {
	name: string,
	path: string,
	type: "File",
	size: (self: FileEntry) -> number,
	read: (self: FileEntry) -> string,
	readbytes: (self: FileEntry, init: number?, final: number?) -> buffer,
	append: (self: FileEntry, content: string | buffer) -> (),
	readlines: (self: FileEntry) -> () -> (number, string),
	is_valid_utf8: (self: FileEntry) -> boolean,
	metadata: (self: FileEntry) -> FsMetadata,
	copy_to: (self: FileEntry, destination: string, relative: boolean?) -> (),
	move_to: (self: FileEntry, destination: string, relative: boolean?) -> (),
	rename: (self: FileEntry, newname: string) -> (),
	remove: (self: FileEntry) -> (),
}

local datetime = require("@std/time/datetime")

type FsMetadata = {
	created_at: datetime.DateTimeUtc?,
	modified_at: datetime.DateTimeUtc?,
	accessed_at: datetime.DateTimeUtc?,
	permissions: {
		readonly: boolean,
		unix_mode: number?,
	}
}

local pathlib = require("@self/path")
fs.path = {} :: pathlib.PathLib

--[=[
	Reads a file from a given string `path`, relative to the current workspace directory.

	Errors if `path` doesn't exist on the file system or if you don't have permission to access that path.
]=]
function fs.readfile(path: string): string
	return nil :: any
end

--[=[
Reads a file to a buffer of bytes, relative to the current working directory.

Errors if `path` doesn't exist on the file system or if you don't have permission to access that path.
]=]
function fs.readbytes(path: string, s: number?, e: number?): buffer
	return nil :: any
end

--[=[
Writes a file given `WriteFileOptions`, which looks like this:
```luau
local writefileoptions: WriteFileOptions = {
	path = "./my/path.txt",
	content = "some content i'm fine overwriting ./my/path.txt with",
	overwrite = true,
}
```

WriteFileOptions.overwrite is true by default (and is optional)
]=]
function fs.writefile(options: WriteFileOptions): ()
	return nil :: any
end

--[=[
Returns a dictionary of paths: Entries you can use to further manipulate and access things on your filesystem.

`path` is relative to your current directory.

Usage:
```luau
for path, entry in fs.entries(".") do
	if entry.type == "File" then
		local file_text = pp(entry:read())
		if entry.extension == "lua" then
			entry:remove()
			fs.writefile {
				path = path:gsub(".lua", ".luau"),
				content = file_text,
			}
		end
	elseif entry.type == "Directory" then
		local file_list = entry:list()
	end
end
```
]=]
function fs.entries(path: string): { [string]: Entry }
	return nil :: any
end


--[=[
Returns an array-like-table containing all the file names/paths in the provided directory.

`path` is relative to your current directory.

Usage:
```lua
local files = fs.list("./typedefs")
```
]=]
function fs.list(path: string): { string }
	return nil :: any
end

--[=[
	Checks if a path exists; errors if permission denied.
	```luau
	local fs = require("@std/fs")
	if fs.exists("./existence") then
		print("yes")
	end
	```
]=]
function fs.exists(path: string): boolean
	return nil :: any
end

export type FindOptions = {
	directory: string
} | {
	file: string
}

--[=[
	Returns an `Entry` if a file/directory is found at the requested path, otherwise returns `nil`.

	Usage:
	```luau
	-- find both files and directories
	local dir = fs.find("./typedefs") 
	if dir.type == "Directory" then
		print(dir:list())
	end

	-- find only files
	local file = fs.find { file = "./typedefs" }
	assert(file == nil) -- ./typedefs is a directory, not a file!!

	-- find only directories
	local dir = fs.find { directory = "./src" }
	```
]=]
function fs.find(path: string | FindOptions): Entry?
	return nil :: any
end

--[=[
	Finds a `DirectoryEntry` at the requested `path` if one exists, otherwise returns nil. Ergonomic alternative to `fs.find { directory: string }`

	## Usage
	```luau
	local tests_dir = fs.dir("./tests")
	if tests_dir then
		for name, entry in tests_dir:entries() do
			-- run test
		end
	end
	```
]=]
function fs.dir(path: string): DirectoryEntry?
	return nil :: any
end
fs.file = {}

function fs.file.from(path: string): FileEntry
	return nil :: any
end
--[=[
Moves or renames a file
]=]
function fs.move(from_path: string, to_path: string): ()
	return nil :: any
end

export type RemoveOptions = RemoveFileOptions | RemoveDirectoryOptions

export type RemoveFileOptions = {
	file: string
}
export type RemoveDirectoryOptions = {
	directory: string,
	force: boolean?, --- this defaults to true
	remove_absolute_path: boolean?, -- only if this is specified and true will you be allowed to remove paths that start in /
}

--[=[
Removes a file/directory relative to your current directory.

Errors if the file/directory you're trying to remove already doesn't exist, or you're trying to remove a non-empty directory
but `RemoveDirectoryOptions.force` is `false`.

Usage:
```lua
if fs.find { file = "existence.jpeg" } then
	fs.remove { file = "existence.jpeg" }
end

-- delet everything (dont try this at home (unless you need to delete everything))
for path, entry in fs.entries(".") do 
	if entry.type == "File" then
		fs.remove { file = path }
	else
		fs.remove {
			directory = path,
			force = true, -- this also recurses; if false, fs.remove will only remove empty directories
		}
	end
end

-- if you need to, for some reason, and you're sure you've verified your input and that user input can't accidentally cause problems here, you can delete dirs by absolute path
fs.remove {
	directory = "/home/myusername/Projects/thing/src",
	force = true, -- this is enabled by default btw
	remove_absolute_path = true,
}

```
]=]
function fs.remove(options: RemoveOptions): ()
	return nil :: any
end

function fs.removefile(path: string): ()
	
end

function fs.removedir(path: string): ()
	
end

export type DirTreeDir = {
	name: string,
	type: "Directory",
	content: { DirTreeDir | DirTreeFile	}?,
}
export type DirTreeFile = {
	name: string,
	type: "File",
	content: string?,
}

export type DirTree = {
	name: string,
	type: "Directory",
	content: { DirTreeDir | DirTreeFile }
}

type CreateOptions = {
	file: string
} | {
	file: {
		name: string,
		content: string
	}
} | {
	directory: string,
} | {
	directory: DirTree
}

--[=[
Creates a new file or directory or directory tree
]=]
function fs.create(options: CreateOptions): DirectoryEntry | FileEntry
	return nil :: any
end

return fs