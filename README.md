# seal, the cutest runtime for the [luau language](https://luau.org)

seal makes writing scripts and programs fun and easy, and runs them seally fast.

## Goals

- Focus on high-level general purpose programming, scripting, and being the best Python alternative for Luau.
- Provide a simple, useful, and expressive API that allows users to get real work done—you should be able to use `seal` right out of the box; common needs should be provided for so you can get straight to working on your project.
- Be helpful and user friendly—if you run into trouble, `seal` should tell you *exactly* where you went wrong with a custom, handcrafted warning or error message.

## Install

If a GitHub release isn't available yet, you can build `seal` with `cargo` by running `cargo build --release`. After it's built, move `seal` to your `PATH` to use it anywhere on your system. Install scripts and releases are planned to make this much easier by 0.1.0.

## Usage

### Setup

Use `seal setup` to generate a new project in your current directory. This autogenerates a `.vscode/settings.json` to configure [Luau Language Server](https://github.com/JohnnyMorganz/luau-lsp) with some default settings, seal's `.typedefs`, a `src` dir, and a `.luaurc` for configuring Luau. Ideally, this means you should be able to just start writing code without much more configuration on your end!

To run a `.luau` file with seal, use `seal <filename_with_ext>` (like `seal ./get_homework.luau`).

To evaluate code with seal, use `seal eval '<string src>'`. `seal eval` comes with the `fs`, `http`, and `process` libs loaded in for convenience. An interactive REPL is planned for the future.

Use `seal run` to run the current project at its entry path (default `./src/main.luau`).

Although seal provides some builtin globals (such as `p`, `pp`, `channel` (in a child thread), and `script`), most features are in the standard library. You can import stdlibs like so:

```luau
local fs = require("@std/fs")
local colors = require("@std/colors") -- (the most important one tbh)

-- some libs are nested:
local input = require("@std/io/input")
```

If you're using VSCode and Luau Language Server, you should be able to see documentation, usage examples, and typedefs for each stdlib by hovering over their variable names in your editor. For convenience, all documentation is located in the `.typedefs` directory generated alongside your project.

### Common tasks

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

### Simple Structured Parallelism

seal is sans-tokio for performance and simplicity, but provides access to Real Rust Threads with a relatively simple, low-level API. Each thread has its own Luau VM, which allows you to execute code in parallel. To send messages between threads, you can use the `:send()` and `:read()` methods located on both `channel`s (child threads) and `JoinHandle`s (parent threads), which seamlessly serialize, transmit, and deserialize Luau data tables between threads (VMs) for you! For better performance, you can use their `bytes` APIs to exchange buffers without the serialization overhead.

Although this style of thread management is definitely less ergonomic than a `task` library, I hope this makes it more reliable and less prone to yields and UB, and is all-around a stable experience.

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
