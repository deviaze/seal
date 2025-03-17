--!nocheck

export type Fs = {
	readfile: (path: string) -> string,
	readbytes: (path: string, target_buffer: buffer, buffer_offset: number, file_offset: number?, count: number?) -> (),
	readlines: (path: string) -> () -> (number, string),

	writefile: (path: string, content: string | buffer) -> (),
	removefile: (path: string) -> (),

	listdir: (path: string, recursive: boolean?) -> { string },
	makedir: (path: string) -> (),

	writetree: (path: string, children: DirectoryChildren) -> (),
	removetree: (path: string) -> (),

	move: (from: string, to: string) -> (),
	copy: (source: string, destination: string) -> (),

	find: (path: string) -> Entry,
	entries: (path: string) -> { [string]: Entry },

	file: {
		from: (path: string) -> FileEntry,
		create: (path: string, content: string) -> FileEntry,
	},

	dir: {
		from: (path: string) -> DirectoryEntry,
		create: (path: string) -> DirectoryEntry,
	},

	path: {
		join: (...string) -> string,
		canonicalize: (path: string) -> string,
		absolutize: (path: string) -> string,
		exists: (path: string) -> boolean,
		parent: (path: string) -> string?,
		child: (path: string) -> string?,
		home: () -> string,
	},

	watch: (paths: string | { string }, recursive: boolean, handler: (Event) -> ()) -> (),
}

type Event = {
	kind: "any" | "access" | "create" | "modify" | "rename" | "remove" | "other",
	paths: { string },
}

type Entry = {
	name: string,
	path: string,
	exists: (self: Entry) -> boolean,
	file: FileEntry?,
	dir: DirectoryEntry?,
	unwrap_file: (self: Entry) -> FileEntry,
	unwrap_dir: (self: Entry) -> DirectoryEntry,
	metadata: (self: Entry) -> FsMetadata,
	move_to: (self: Entry, destination: string) -> (),
	copy_to: (self: Entry, destination: string) -> (),
	rename: (self: Entry, name: string) -> (),
	remove: (self: Entry) -> (),
}

type FileEntry = {
	name: string,
	path: string,
	type: "File",
	size: (self: FileEntry) -> number,
	read: (self: FileEntry) -> string,
	readbytes: (self: FileEntry, target_buffer: buffer, buffer_offset: number, file_offset: number?, count: number?) -> (),
	append: (self: FileEntry, content: string | buffer) -> (),
	readlines: (self: FileEntry) -> () -> (number, string),
	is_valid_utf8: (self: FileEntry) -> boolean,
	metadata: (self: FileEntry) -> FsMetadata,
	copy_to: (self: FileEntry, destination: string) -> (),
	move_to: (self: FileEntry, destination: string) -> (),
	rename: (self: FileEntry, name: string) -> (),
	remove: (self: FileEntry) -> (),
}

type DirectoryEntry = {
	name: string,
	path: string,
	type: "Directory",
	list: (self: DirectoryEntry, recursive: boolean?) -> { string },
	entries: (self: DirectoryEntry) -> { [string]: Entry },
	find: (self: DirectoryEntry, path: string) -> Entry,
	expect_file: (self: DirectoryEntry, filename: string) -> FileEntry,
	expect_dir: (self: DirectoryEntry, dirname: string) -> DirectoryEntry,
	create_file: (self: DirectoryEntry, filename: string, content: string) -> FileEntry,
	create_dir: (self: DirectoryEntry, dirname: string) -> DirectoryEntry,
	write_children: (self: DirectoryEntry, children: DirectoryChildren) -> (),
	metadata: (self: DirectoryEntry) -> FsMetadata,
	copy_to: (self: DirectoryEntry, destination: string) -> (),
	move_to: (self: DirectoryEntry, destination: string) -> (),
	rename: (self: DirectoryEntry, name: string) -> (),
	remove: (self: DirectoryEntry) -> (),
}

type FsMetadata = {
	created_at: datetime.DateTimeUtc?,
	modified_at: datetime.DateTimeUtc?,
	accessed_at: datetime.DateTimeUtc?,

	permissions: {
		readonly: boolean,
		unix_mode: number?,
	},
}

type DirectoryChildren = {
	files: {
		[number]: {
			name: string,
			content: string,
		},
	},
	directories: {
		[number]: {
			name: string,
			children: DirectoryChildren?,
		},
	},
}

return {} :: Fs
