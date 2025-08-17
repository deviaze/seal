export type PathLib = {
	--[=[
		Joins path components together in a cross-platform-safe manner.

		The default separator is `/`, except when dealing with absolute paths on Windows.

		On Windows, pass `.\` as the first component to `path.join` to use `\` in relative paths.

		## Usage
		```luau
		local srcpath = path.join(path.cwd(), "src")
		local main_luau = path.join(srcpath, "main.luau")
		local main_content = fs.readfile(main_luau)

		local otherfile_in_script_dir = path.join(script:parent(), "otherfile.txt")
		```
	]=]
	join: (...string) -> string,
	--[=[
		Checks if `path` exists on the filesystem using Rust's `std::fs::exists`.
		
		Note this function is ***not* TOCTOU (Time Of Check to Time Of Use)-safe**!
			
		In security-critical applications, use relatively error-safe functions like `fs.file.try_read`, `fs.file.try_write`, etc., which allow you to
		handle cases like `NotFound` and `PermissionDenied` without wrapping error-throwing functions like `fs.readbytes` in a pcall.
	]=]
	exists: (path: string) -> boolean,
	--[=[
		Returns the canonical (absolute) form of `path` using Rust's `std::fs::canonicalize`, resolving symlinks and intermediate components.

		Errors if the requested path doesn't exist on the filesystem or is invalid.
	]=]
	canonicalize: (path: string) -> string,
	--[=[
		Returns the absolute path of `path` without checking the filesystem.

		Use this function if your path may or may not exist (yet).
	]=]
	absolutize: (path: string) -> string,
	--[=[
		Returns a normalized (cleaned) version of `path` with a consistent path separator and with duplicate separators and unneeded relative path symbol removed.

		By default, uses '/' as the path separator unless `path` is a Windows-style absolute path, in which case it'll use a backslash instead.

		```luau
		local mixed_path = [[./hi/im/a\./file.txt]]
		print(path.normalize(mixed_path)) --> "./hi/im/a/file.txt"

		-- absolute paths on windows use \
		local windows_path = [[C:\Users\Example\Documents\project/main.luau]]
		print(path.normalize(windows_path)) --> "C:\Users\Example\Documents\project\main.luau"

		-- paths with redundant separators get cleaned
		local redundant_separators = [[C:\\Users\\Example//Documents////project\main.luau]]
		print(path.normalize(redundant_separators)) --> "C:\Users\Example\Documents\project\main.luau"
		```

		For Windows-style absolute paths, `path.normalize` handles both drive letter paths like `"C:\Users\Username\Documents\..."`
		as well as UNC paths like `"\\network\share\text.txt"` or `"\\?\wsl\mnt\..."`.
	]=]
	normalize: (path: string) -> string,
	--[=[
		Returns the path of the parent directory `n` (default = 1) parents to the left of `path`

		## Usage
		```luau
		local fs = require("@std/fs")
		local env = require("@std/env")
		local path = fs.path

		local cwd = env.current_working_directory
		local parent_dir = path.parent(cwd)		
		```
	]=]
	parent: (path: string, n: number?) -> string?,
	--- the farthest child/leaf/node of the path, ex. `path.child("./src/main.luau") == "main.luau"`
	child: (path: string) -> string?,
	--- returns the user's home directory, also known as `~`
	home: () -> string,
	--- returns the current working directory, errors if not found or is invalid utf-8
	cwd: () -> string,
}

return {} :: PathLib