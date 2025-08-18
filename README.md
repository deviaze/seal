<!-- markdownlint-disable MD033 -->

# seal, the cutest runtime for the [luau language](https://luau.org)

~~*seal* makes writing scripts and programs fun and easy, and runs them seally fast.~~

*seal* is a highly reliable cross-platform scripting runtime.

## Features

- Expressive filesystem library `@std/fs`, with APIs for proper error handling, an integrated path library that handles cross-platform edgecases, support for partially reading files (into buffers), reading extremely large files line-by-line, reading and writing entire directory trees, filesystem watching, etc.
- User-defined parallelism with `@std/thread` featuring communication via message passing and automatic table serialization.
- Process library with both blocking and nonblocking `ChildProcess` spawning APIs, including iterating over a running child process' `stdout` and `stderr`, reading streams line by line, reading until a specific token is reached, etc.
- UTF-8 and grapheme-aware string library (`@str`) with grapheme-aware iteration and grapheme-aware string splitting as well as many convenience functions.
- UX and convenience features across the API, including automatic JSON serialization of tables passed to `@std/net/http` APIs alongside the relevant headers.
- Some cryptography and password hashing functions (built into seal for security reasons).
- Many other standard libraries!

## Upcoming features (0.1.0 -> 0.2.0 roadmap)

- Integrated project compilation to static executables for portability (with Darklua).
- Integrated webview bindings/UI library for GUI scripts.
- Automation:
  - Keyboard rebinding bindings to write custom keyboard mapping layers for all desktop platforms.
  - Mouse/keyboard automation if possible.

## Goals

<!-- - Focus on high-level general purpose programming, scripting, and being the best Python alternative for Luau.
- Provide a simple, useful, and expressive API that allows users to get real work done—you should be able to use `seal` right out of the box; common needs should be provided for so you can get straight to working on your project.
- Be helpful and user friendly—if you run into trouble, `seal` should tell you *exactly* where you went wrong with a custom, handcrafted warning or error message. -->

- Focus on high-level scripting, light general purpose programming, and being the best Python alternative for Luau.
- Provide a simple, useful, and (yet) expressive standard library that allows users to get real work done—*seal* should work right of out the box so you can get straight to working on your script, shim, or project.
- Be extremely helpful and user friendly. You should know exactly how to use *seal* by reading its docs, and when you inevitably run into trouble, *seal* should tell you *exactly* what went wrong with a custom, handcrafted recommendation, warning or error message.
- *seal* should integrate well with tooling, other languages, and other runtimes. Setting up new projects should be instantaneous, and adding *seal* to existing (*seal* and non-*seal*) projects should be just as easy.
- Reliability and transparency. *seal* should *just work* and never cause unexpected blocks, deadlocks, panics, nor unexpected behavior. Standard library behavior must be well documented, and all of *seal*'s internals should be readily accessible so *seal* remains easy to understand, hackable, customizable, and fixable by its users.

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

Run `seal project` to generate a **Project** codebase at your current directory.

### Scripts

Use a **Script** codebase when you want to add *seal* to an existing project to run build or glue scripts, without making *seal* the whole point of your project. This option generates a `.seal` directory locally for seal configuration, but will otherwise link to user-wide typedefs in `~/.seal/typedefs/*`. `.vscode/settings.json` and `.luaurc`s will also be created or updated to include *seal*'s typedefs and default config.

Run `seal script` to add a **Script** codebase to your current directory.

### Using Projects/Scripts

Both Project and Script codebases should have a `.seal/config.luau` file, which you can modify to set a codebase entry path, test runner path, etc.

To run a codebase at its entry path, use `seal run`. Note this command is similar to `cargo run` in Rust, and isn't used to run single files.

The general setup for a project should follow:

1. `mkdir/md ProjectName`
2. `cd Pro-` (tab autocomplete)
3. `seal sp` (short form for `seal setup project`)
4. `code .` -- or `zeditor .`

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

If you're using VSCode and Luau Language Server, you should be able to see documentation, usage examples, and typedefs for each stdlib by hovering over their variable names in your editor. For convenience, in **Project** codebases, all documentation is located in the `.seal/typedefs` directory generated alongside your project.

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
        local recursive_list = table.concat(entry:list(true))
        print(`directory at {colors.blue(`'{entry_path}'`)} has these entries, recursively: {recursive_list}'`)
    end
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

Although this style of thread management is can be less ergonomic than a `task` library, I hope this makes it more reliable and less prone to yields and UB, and is all-around a stable experience.

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

- Fully featured standard library for all usecases: `seal` is primarily suited for high level scripting and general purpose programming. We don't want to add every single hash algorithm, nor bind to every single part of Rust's standard library—providing too many options might end up confusing to the average user.
- Top tier performance: although `seal` is pretty fast, `mlua` isn't the fastest way to use Luau; runtimes like `Zune` and `lute` may be faster than `seal`. On the other hand, because `seal` doesn't have any tokio or async overhead, its standard library should be faster than `Lune`'s, and because of its parallelism model, multithreaded programs in `seal` should be more stable than async programs in `Lune`.
