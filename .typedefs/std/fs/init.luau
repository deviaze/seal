
local dirlib = require("./dir")
local pathlib = require("./path")
local filelib = require("./file")
local common_types = require("./_types")

export type fs = {
	readfile: (path: string) -> string,
	readbytes: (path: string, file_offset: number?, count: number?, target_buffer: buffer?, buffer_offset: number?) -> buffer,
	readlines: (path: string) -> () -> (number, string),

	writefile: (path: string, content: string | buffer) -> (),
	removefile: (path: string) -> (),

	readtree: (path: string) -> DirectoryTree,
	writetree: (path: string, tree: TreeBuilder | DirectoryTree) -> (),
	removetree: (path: string) -> (),

	makedir: (path: string, options: { create_missing: boolean?, error_if_exists: boolean? }?) -> boolean,
	listdir: (path: string, recursive: boolean?) -> { string },

	move: (from: string, to: string) -> (),
	copy: (source: string, destination: string) -> (),

	find: (path: string, options: { follow_symlinks: boolean?, error_if_permission_denied: boolean? }?) -> FindResult,
	entries: (path: string) -> { [string]: Entry },

	file: filelib.FileLib,
	dir: dirlib.DirLib,
	path:  pathlib.PathLib,

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