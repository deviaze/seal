--!strict
export type RunResult = ({
	ok: true,
	out: string,
	stdout: string,
	stderr: string,
} | {
	ok: false,
	err: string,
	stdout: string,
	stderr: string,
}) & {
	unwrap: (self: RunResult) -> string,
	unwrap_or: (self: RunResult, default: string | (result: RunResult) -> string) -> string
}

export type RunOptions = {
	program: string,
	args: { string }?,
	--- specify a shell to run the program with; otherwise runs it as a bare process with no shell
	shell: string?,
}

type ChildProcessStdout = {
	read: (self: ChildProcessStdout, buffer_size: number?) -> string?,
	lines: (self: ChildProcessStdout) -> (() -> string)
}

type ChildProcessStderr = {
	read: (self: ChildProcessStderr, buffer_size: number?) -> string?,
	lines: (self: ChildProcessStderr) -> (() -> string)
}

type ChildProcessStdin = {
	write: (self: ChildProcessStdin, data: string) -> nil
}

export type ChildProcess = {
	id: number,
	alive: (self: ChildProcess) -> boolean,
	kill: (self: ChildProcess) -> nil,
	stdout: ChildProcessStdout,
	stderr: ChildProcessStderr,
	stdin: ChildProcessStdin,
}

type process = {
	--[=[
	Runs a program, yields until it completes, and returns its results.

	Takes a RunOptions table:
	```luau
	type RunOptions = {
		program: string,
		--- optional args you want to pass
		args: { string }?,
		--- the name or path of the shell, omit to run without shell
		shell: string? 
	}
	```

	### Usage
	```luau
	local process = require("@std/process")
	local result = process.run {
		program = "lune",
		args = {"run", somefile},
	}
	if result.ok then
		print(result.stdout)
	end
	```
	]=]
	run: (options: RunOptions) -> RunResult,
	--[=[
	Launches a shell command in a child process.

	Uses the same shell you're using to run `seal` (so your aliases should available)
	### Usage
	```luau
	local process = require("@std/process")
	local file_stuff: {string} = process.shell("ls -l")
		:unwrap()
		:split("\n")
	print(file_stuff)
	```
	]=]
	shell: (command: string) -> RunResult,
	--[=[
	Spawns a long-running process in a non-blocking manner, returns a `ChildProcess` that contains handles to the spawned process' stdout, stderr, and stdin.

	## Usage
	```luau
	local process = require("@std/process")
	local child = process.spawn({
		program = "someutil --watch",
		shell = "sh",
	})
	
	for line in child.stdout:lines() do
		local thing_changed = line:match("([%w]+) changed!")
		print(`Change detected: {thing_changed}`)
	end
	```
	]=]
	spawn: (options: RunOptions) -> ChildProcess,
	setexitcallback: ((number) -> ()) -> (),
	exit: (code: number?) -> never,
}

return {} :: process
