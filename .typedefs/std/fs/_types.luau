-- common types that we don't want cyclic require errors on
local datetime = require("@std/time/datetime")

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
	find: (self: DirectoryEntry, name: string, options: { follow_symlinks: boolean?, error_if_permission_denied: boolean? }?) -> FindResult,
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
	type: "File" | "Directory" | "Symlink" | "NotFound" | "PermissionDenied",
	exists: (self: FindResult) -> boolean,
	try_file: (self: FindResult) -> FileEntry?,
	try_dir: (self: FindResult) -> DirectoryEntry?,
	unwrap_file: (self: FindResult) -> FileEntry,
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