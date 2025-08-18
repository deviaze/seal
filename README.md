<!-- markdownlint-disable MD033 -->

# seal, the cutest runtime for the [luau language](https://luau.org)

~~*seal* makes writing scripts and programs fun and easy, and runs them seally fast.~~

*seal* is a highly reliable cross-platform scripting runtime.

## Goals

- Focus on high-level scripting, light general purpose programming, and being the best Python alternative for Luau.
- Provide a simple, useful, and expressive standard library that allows users to get real work done—*seal* should work right of out the box so you can get straight into working on your script, shim, or project.
- Be extremely helpful and user friendly. If run into trouble, *seal* should tell you *exactly* what went wrong with a custom, handcrafted recommendation, warning or error message.
- Reliability and transparency. *seal* should *just work* and never cause unexpected blocks, deadlocks, panics, nor unexpected behavior. Standard library behavior must be well documented, and all of *seal*'s internals should be readily accessible so *seal* remains easy to understand, hackable, customizable, and fixable by its users.
<!-- - *seal* should integrate well with tooling, other languages, and other runtimes. Setting up new projects should be instantaneous, and adding *seal* to existing (*seal* and non-*seal*) projects should be just as easy. -->

## Features
<!-- 
flowery version
- An extensive, expressive, easy to use, and well documented standard library, covering many usecases from simple filesystem operations (`@std/fs`) to password hashing (`@std/crypt/password`).
- A filesystem library API with support for partial reads, reading files line-by-line, granular error handling, filesystem watching, with an integrated path library that handles cross-platform edgecases, etc.
- A process library (`@std/process`) for spawning platform-aware shell commands, executing programs, and running long-running, nonblocking `ChildProcesses` with nonblocking access to their stdout, stderr, and stdin.
- User-defined parallelism with `@std/thread` featuring communication via message passing and automatic table serialization, backed by Rust threads and crossbeam-channel.
- A UTF-8 and grapheme-aware string library (`@std/str`) with extremely fast grapheme-aware string splitting, and many other convenience functions.
- Many other standard libraries! -->

- Batteries-included, immediate project setup for both standalone projects and extension scripts.
- Easy-to-use standard library covering usecases from filesystem management to password hashing.
- User-defined multithreading with automatic serialization and message passing.
- Strings extension library with full Unicode graphemes support.

... and more; see the [full list of features](docs/index.md).

## Upcoming features (0.1.0 -> 0.2.0 roadmap)

- Integrated standalone project compilation for portability (with Darklua).
- Integrated webview bindings/UI library for GUI scripts.
- Automation:
  - Keyboard rebinding bindings to write custom keyboard mapping layers for all desktop platforms.
  - Mouse/keyboard automation if possible.

## Install and Setup

### Install

To use *seal*, you need 3 things:

1. A text editor that supports the Luau Language Server, such as `VSCode`, `Zed`, or `nvim`.
2. [Luau Language Server](https://github.com/JohnnyMorganz/luau-lsp), which provides amazing inline documentation, static analysis and typechecking support, and diagnostics for the Luau programming language. *seal* automatically sets up everything for Luau Language Server when you run `seal setup/project/script/custom`.
3. The *seal* executable, which you can find packaged for your platform here: <https://github.com/deviaze/seal/releases/latest>.

### Setup

1. Make *seal* available in your `PATH` so you can use it in a terminal.

<details>
<summary>How to add seal to PATH?</summary>

Option 1 - using *seal*

1. Save this *seal* script to your Downloads folder: [seal_install.luau](examples/seal_install.luau)
2. Read it so you know how it works! Or modify the path so it moves seal where you want it to.
3. Open your Downloads folder in your terminal and run `./seal ./seal_install.luau`
4. On Windows, add the `~\.local\bin` path to your `$PROFILE` file with the instructions provided.
5. Open a new terminal and make sure `seal --help` works.

Option 2 - Windows Terminal on Windows:

1. Open Windows Terminal (PowerShell)
2. Move `seal` somewhere permanent like `C:\Users\<USERNAME>\.local\bin`:
   1. Open your Downloads folder (`cd "~\Downloads"` or `cd "~\OneDrive\Downloads"`) and run `mv .\seal.exe "~\.local\bin\seal.exe"`
3. Run `code $PROFILE` to open your powershell profile in vscode.
4. Add `$env:Path += ";C:\Users\<USERNAME>\.local\bin"` near the bottom or wherever you add your paths.
5. Close and reopen your Windows Terminal and run `seal --help` to make sure seal is available.

</details>

## Usage

*seal* codebases can be either *Projects*, *Scripts*, or single files.

### Projects

Use a **Project** codebase when you want to use *seal* as the primary runtime for your project; this option will generate `.seal` directory, all typedefs locally for easy portability (and soon, compilation), a `src` dir, a `.luaurc`, a `.vscode/settings.json`, and will start a new `git` repository if one doesn't already exist.

Run `seal setup project` or `seal sp` to generate a **Project** codebase at your current directory.

### Scripts

Use a **Script** codebase when you want to add *seal* to an existing project to run build or glue scripts, without making *seal* the whole point of your project. This option generates a `.seal` directory locally for seal configuration, but will otherwise link to user-wide typedefs in `~/.seal/typedefs/*`. `.vscode/settings.json` and `.luaurc`s will also be created or updated to include *seal*'s typedefs and default config.

Run `seal setup script` or `seal ss` to add a **Script** codebase to your current directory.

### Using Projects/Scripts

Both Project and Script codebases should have a `.seal/config.luau` file, which you can modify to set a codebase entry path, test runner path, etc.

To run a codebase at its entry path, use `seal run` or `seal r`. Note this command is similar to `cargo run` in Rust, and isn't used to run single files.

The general setup for a project should follow:

1. Open a terminal
2. `mkdir/md ProjectName`
3. `cd ProjectName`
4. `seal sp` (setup a project codebase)
5. `code .` or `zeditor .`

Automatic setup for Zed is not fully ready yet, but all the other settings are available for config when you run `seal setup custom`.

### Running single files

To run a `.luau` file with seal, use `seal <filename_with_ext>` (like `seal ./get_the_endpoint.luau`).

To evaluate code with seal, use `seal eval '<string src>'`. `seal eval` comes with the `fs`, `http`, and `process` libs loaded in for convenience. An interactive REPL is planned for the future.

### Programming and Standard Library

Although seal provides some builtin globals (such as `p`, `pp`, `channel` (in a child thread), and `script`), most features are in the standard library. You can import stdlibs like so:

```luau
local fs = require("@std/fs")
local colors = require("@std/colors") -- (the most important one tbh)

-- some libs are nested:
local input = require("@std/io/input")
```

Using Luau Language Server, you should be able to see documentation, usage examples, and typedefs for each standard library type/table/function by hovering over their variable names in your editor. For convenience, in **Project** codebases, all documentation is located in the `.seal/typedefs` directory generated alongside your project.

### Common tasks

<details>
<summary> Read and write files/directories </summary>

#### Read and write files/directories

```luau
local fs = require("@std/fs")
local path = fs.path

-- read files
local content = fs.readfile("myfile.txt")

-- write a file from string (or buffer!)
local seally_path = path.join(path.cwd(), "seally.txt")
fs.writefile(seally_path, "did you know seals can bark?")

-- remove it
fs.removefile(seally_path)

-- make a new empty directory
fs.makedir("./src")
-- write a new directory tree
fs.writetree("./tests", fs.tree()
    :with_file("run_tests.luau", run_tests_src)
    :with_tree("cases", fs.tree()
        :with_file("case1", cases[1])
    )
)
-- remove both
fs.removetree("./src"); fs.removetree("./tests")
```

#### Iterate through a directory's entries

```luau
local entries = fs.entries(path.join(script:parent(), "other_dir"))
for entry_path, entry in entries do
    if entry.type == "File" then
        print(`file at '{entry_path}' says {entry:read()}!`)
    elseif entry.type == "Directory" then
        local recursive_list = entry:list(true) -- you can also add a filter function if you want
        print(`directory at {colors.blue(`'{entry_path}'`)} has these entries, recursively:`)
        print(recursive_list)
    end
end
```

#### Check if a file exists

```luau
-- because you want to read it
local content, result = fs.file.try_read(mypath)
if content then
    print(content)
elseif result == "NotFound" then
    print(`{mypath} not found`)
else
    warn(`unexpected error reading {mypath}: {result}`)
end

-- because you just want to make sure it exists
if fs.path.exists(mypath) then
    print("yes it exists")
end
```

</details>

<!-- #### Read and write files/directories -->

<details>
<summary> Send http requests </summary>

#### Send http requests

```luau
local http = require("@std/net/http")

local seal_data = http.get("https://sealfinder.net/api/get"):unwrap_json()
local post_response = http.post {
    url = "https://mycatlist.me/api/add_cat/post",
    headers = {
        Authorization = `Bearer {TOKEN}`,
    },
    body = {
        name = "Taz",
        age = 12,
    }, -- pass a table? seal serializes it for you (and sets Content-Type: application/json)!
}
```

</details>

<details>
<summary> Spawning processes </summary>

#### Spawning processes ~~(ffi at home)~~

```luau
local process = require("@std/process")
-- run a shell command
local output = process.shell("seal ./cats.luau"):unwrap()

-- run a program directly (waits til it completes)
local result = process.run {
    program = "seal",
    args = { "./cats.luau" },
}:unwrap()

-- spawn a program as a long-running child process
local child = process.spawn {
    program = "somewatcher",
    args = { "./somefile.json" }
}
if you_want_to_block_main_thread then
    for line in child.stdout:lines() do
        print(line)
    end
else
    local text: string? = child.stdout:read(24)
end
```

</details>

### Simple Structured Parallelism

seal is sans-tokio for performance and simplicity, but provides access to Real Rust Threads with a relatively simple, low-level API. Each thread has its own Luau VM, which allows you to execute code in parallel. To send messages between threads, you can use the `:send()` and `:read()` methods located on both `channel`s (child threads) and `JoinHandle`s (parent threads), which seamlessly serialize, transmit, and deserialize Luau data tables between threads (VMs) for you! For better performance, you can use their `bytes` APIs to exchange buffers without the serialization overhead.

Although this style of thread management can be less ergonomic than a `task` library or implicit futures everywhere, I hope this makes it more reliable and less prone to yields and UB, and is all-around a stable experience.

```luau
-- parent.luau
local thread = require("@std/thread")

local handle = thread.spawn {
    path = "./child.luau",
    data = {
        url = "https://example.net",
    }
}
 -- do something else
local res = handle:read_await()

handle:join() -- don't forget to join your handles!
```

Child threads have a global `channel` exposed, which you can use to send data to the main thread:

```luau
-- child.luau
local http = require("@std/net/http")
if channel then
    local data = channel.data :: { url: string }
    local response = http.get(data.url):unwrap_json()
    channel:send(response)
end
```

### Non-goals

- Fully featured standard library for all usecases: `seal` is primarily suited for high level scripting and light general purpose programming. We don't want to add every single hash algorithm, nor bind to every single part of Rust's standard library—providing too many options might end up confusing to the average user. Use [Zune](<https://github.com/Scythe-Technology/zune>) instead if you need lower level bindings.
- Async webservers. *seal* is not an async runtime and thus cannot scale up enough for webservers, but if you want to write one (or a Discord bot) in Luau and need async, I highly recommend using [Zune](<https://github.com/Scythe-Technology/zune>) instead.
- Premature optimization. Although *seal* is very fast, it might not be the absolute fastest way to use Luau due to `mluau`'s slight safety overhead. On the other hand, because `seal` doesn't have any tokio or async overhead, its standard library should be faster than `Lune`'s, and because of its parallelism model, true multithreaded programs in `seal` should be more stable than programs that rely on Lune's `task` library and IO.

## Contributing

I would greatly appreciate any contributions and feedback, including issues, PRs, or even messages on Discord saying "hey can you add this to seal"!

See the [contribution instructions](CONTRIBUTING.md) if you'd like to contribute to seal's codebase :3

If you find a bug in seal, please make a bug report issue on GitHub and ping me on Discord.
