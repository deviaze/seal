# seal, the cutest ~~runtime~~ for the [luau language](https://luau.org)

seal makes writing scripts and programs fun and easy, and runs them seally fast.

## Goals

- General purpose scripting, file/directory management, http requests, and being the best shell-script alternative for Luau.
- Provide a simple, useful, and expressive API that allows users to get real work done.
- Helpful and user friendly - you should be able to use `seal` right out of the box with as minimal hassle as possible. If you run into trouble, `seal` should tell you *exactly* where you went wrong with a custom, handcrafted warning or error message.

## Usage

### Install

If a release isn't available yet, you can build `seal` with `cargo` by running `cargo build --release`. After it's built, move `seal` to your `$PATH` to use it anywhere on your system. Install scripts and releases are planned to make this much easier by 0.1.0.

### Setup

`seal` doesn't spawn any top-level settings or type definitions, instead, it adds its settings and type definitions directly to your project's workspace for easy reference.

Use `seal setup` to generate a new project in your current directory. This autogenerates a `.vscode/settings.json` to configure [Luau Language Server](https://github.com/JohnnyMorganz/luau-lsp) with some default settings, seal's `.typedefs`, a `src` dir, and a `.luaurc` for configuring Luau. Ideally, this means you should be able to just start writing code without much more configuration on your end!

To run a `.luau` file with seal, use `seal <filename_with_ext>`. To evaluate code within seal (with fs/net/process libs already loaded in), use `seal eval '<string src>'`. Use `seal run` to run the project in your current workspace (runs the project's entry file (usually `main.luau`)).

Although seal provides some builtin globals, most features are in the standard library. You can import stdlibs like so:

```luau
local fs = require("@std/fs")
local http = require("@std/net/http")
local process = require("@std/process")
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
-- read half the file
local verybigfile = fs.file.from("verybig.txt")
local half_the_file = buffer.tostring(verybigfile:readbytes(0, verybigfile.size // 2))

-- write a file from string (or buffer!)
local seally_path = path.join(path.cwd(), "seally.txt")
fs.writefile(seally_path, "did you know seals can bark?")

-- iterate through a directory
local other_dir = path.join(script:parent(), "otherdir")
for entry_path, entry in fs.entries(other_dir) do
    if entry.type == "File" then
        print(`file at '{entry_path}' says {entry:read()}!`)
    elseif entry.type == "Directory" then
        local recursive_list = table.concat(entry:list(true))
        print(`directory at '{entry_path} has these entries, recursively: {recursive_list}'`)
    end
end

-- make a directory
fs.makedir("./src")
-- write a directory tree
fs.writetree("./tests", fs.tree()
    :with_file("run_tests.luau", run_all_tests_src)
    :with_tree("cases", fs.tree()
        :with_file("case1.luau", cases[1])
        :with_file("case2.luau", cases[2])
    )
)
-- remove both
fs.removetree("./src")
fs.removetree("./tests")
```

### Send http requests

```luau
local http = require("@std/net/http")

local json = http.get({
    url = "sealfinder.net/api/get",
}):unwrap_json()
```

### Spawning processes (ffi at home)

```luau
local process = require("@std/process")
-- run a shell command
local output = process.shell("seal ./cats.luau"):unwrap()

-- run a program properly (waits til it completes)
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

seal is fully sans-tokio and async-less for performance, efficiency, and simplicity.

But, you want > 1 thing to happen at once nonetheless? seal provides access to Real Rust Threads with a relatively simple, low-level API. Each thread has its own Luau VM, which allows you to execute code in parallel. To send messages between threads, you can use the `:send()` and `:read()` methods located on both `channel`s (child threads) and `JoinHandle`s (parent threads), which seamlessly serialize, transmit, and deserialize Luau data tables between threads (VMs) for you! For better performance, you can use their `bytes` APIs to exchange buffers without the serialization overhead.

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
    local response = http.get({ url = data.url }):unwrap_json()
    channel:send(response)
end
```
