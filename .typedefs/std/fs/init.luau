local pathlib = require("./path")
local datetime = require("@std/time/datetime")

export type fs = {
	readfile: (path: string) -> string,
	readbytes: (path: string, file_offset: number?, count: number?, target_buffer: buffer?, buffer_offset: number?) -> buffer,
	readlines: (path: string) -> () -> (number, string),

	writefile: (path: string, content: string | buffer) -> (),
	removefile: (path: string) -> (),

	readtree: (path: string) -> DirectoryTree,
	writetree: (path: string, tree: TreeBuilder | DirectoryTree) -> (),
	removetree: (path: string) -> (),

	makedir: (path: string, create_missing: boolean?) -> (),
	listdir: (path: string, recursive: boolean?) -> { string },

	move: (from: string, to: string) -> (),
	copy: (source: string, destination: string) -> (),

	find: (path: string, follow_symlinks: boolean?) -> FindResult,
	entries: (path: string) -> { [string]: Entry },

	file: setmetatable<{
		from: (path: string) -> FileEntry,
		build: (name: string, content: string) -> FileBuilder,
		create: (path: string) -> FileEntry,
	}, {
		__call: (self: any, path: string) -> FileEntry?,
	}>,

	dir: setmetatable<{
		from: (path: string) -> DirectoryEntry,
		build: (name: string, tree: DirectoryTree) -> DirectoryBuilder,
		create: (path: string) -> DirectoryEntry,
	}, {
		__call: (self: any, path: string) -> DirectoryEntry?,
	}>,

	tree: () -> TreeBuilder,

	path: pathlib.PathLib,
}

export type TreeBuilder = {
    inner: DirectoryTree,
    with_file: (self: TreeBuilder, name: string, content: string) -> TreeBuilder,
    with_tree: (self: TreeBuilder, name: string, builder: TreeBuilder) -> TreeBuilder,
}

export type FileEntry = {
	name: string,
	path: string,
	type: "File",
	read: (self: FileEntry) -> string,
	size: (self: FileEntry) -> number,
	readlines: (self: FileEntry) -> () -> (number, string),
	readbytes: (self: FileEntry, file_offset: number?, count: number?, target_buffer: buffer?, buffer_offset: number?) -> buffer,
	append: (self: FileEntry, content: buffer | string) -> (),
	is_valid_utf8: (self: FileEntry) -> boolean,
	metadata: (self: FileEntry) -> FsMetadata,
	copy_to: (self: FileEntry, to: string) -> (),
	move_to: (self: FileEntry, to: string) -> (),
	rename: (self: FileEntry, name: string) -> (),
	remove: (self: FileEntry) -> (),
}

export type DirectoryEntry = {
	name: string,
	path: string,
	type: "Directory",
	list: (self: DirectoryEntry, recursive: boolean?) -> { string },
	find: (self: DirectoryEntry, name: string, follow_symlinks: boolean?) -> FindResult,
	entries: (self: DirectoryEntry) -> { [string]: Entry },
	expect_file: (self: DirectoryEntry, name: string) -> FileEntry,
	expect_dir: (self: DirectoryEntry, name: string) -> DirectoryEntry,
	add_file: (self: DirectoryEntry, name: string, content: string) -> DirectoryEntry,
	add_tree: (self: DirectoryEntry, name: string, builder: TreeBuilder) -> DirectoryEntry,
	metadata: (self: DirectoryEntry) -> FsMetadata,
	copy_to: (self: DirectoryEntry, to: string) -> (),
	move_to: (self: DirectoryEntry, to: string) -> (),
	rename: (self: DirectoryEntry, name: string) -> (),
	remove: (self: DirectoryEntry) -> (),
}

export type Entry = FileEntry | DirectoryEntry

export type FsMetadata = {
	created_at: datetime.DateTimeUtc?,
	modified_at: datetime.DateTimeUtc?,
	accessed_at: datetime.DateTimeUtc?,
	permissions: {
		readonly: boolean,
		unix_mode: number?,
	}
}

export type FindResult = { -- can't make this type more accurate w/ unions bc it breaks (new) typesolver
	ok: boolean,
	path: string,
	exists: (self: FindResult) -> boolean,
	file: FileEntry?,
	dir: DirectoryEntry?,
	retry_file: (self: FindResult) -> FileEntry?,
	retry_dir: (self: FindResult) -> DirectoryEntry?,
	unwrap_file: (self: FindResult) -> FileEntry,
	unwrap_dir: (self: FindResult) -> DirectoryEntry,
	err: "PermissionDenied"?,
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

return {} :: fs