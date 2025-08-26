<!-- markdownlint-disable MD033 -->
<!-- 
flowery version
- An extensive, expressive, easy to use, and well documented standard library, covering many usecases from simple filesystem operations (`@std/fs`) to password hashing (`@std/crypt/password`).
- A filesystem library API with support for partial reads, reading files line-by-line, granular error handling, filesystem watching, with an integrated path library that handles cross-platform edgecases, etc.
- A process library (`@std/process`) for spawning platform-aware shell commands, executing programs, and running long-running, nonblocking `ChildProcesses` with nonblocking access to their stdout, stderr, and stdin.
- User-defined parallelism with `@std/thread` featuring communication via message passing and automatic table serialization, backed by Rust threads and crossbeam-channel.
- A UTF-8 and grapheme-aware string library (`@std/str`) with extremely fast grapheme-aware string splitting, and many other convenience functions.
- Many other standard libraries! -->
# Programming and Standard Library

Luau is a simple, dependable, and extremely fast scripting language with strict typechecking and tooling support.

Although seal provides some builtin globals (such as `p`, `pp`, `channel` (in a child thread), and `script`), most features are in the standard library. You can import stdlibs like so:

```luau
local fs = require("@std/fs")
local colors = require("@std/colors") -- (the most important one tbh)

-- some libs are nested:
local input = require("@std/io/input")
```

Using Luau Language Server, you should be able to see documentation, usage examples, and typedefs for each standard library type/table/function by hovering over their variable names in your editor. For convenience, in **Project** codebases, all documentation is located in the `.seal/typedefs` directory generated alongside your project.

## Common tasks

<details>
<summary> Read and write files/directories </summary>

### Read and write files/directories

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
