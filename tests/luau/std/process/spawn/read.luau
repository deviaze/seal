--!nolint LocalShadow
local fs = require("@std/fs")
local env = require("@std/env")
local str = require("@std/str")
local time = require("@std/time")
local path = require("@std/fs/path")
local process = require("@std/process")

local cheese = require("@tests/cheese")

local function spawn_long_waiter(): process.ChildProcess
    return process.spawn {
        program = env.executable_path,
        args = { path.join(script:parent(), "long_waiter.luau") }
    }
end

local longrunning_path = path.join(script:parent(), "longrunning.luau")

local function spawn_longrunning(): process.ChildProcess
    local child = process.spawn {
        program = env.executable_path,
        args = { longrunning_path, "85", "50" },
    }
    return child
end

local function spawn_poem(stdout_capacity: number, stderr_capacity: number, stdout_truncate: "front" | "back"): (process.ChildProcess, string)
    local src = [[
    local fs = require("@std/fs")
    local time = require("@std/time")
    local lines = {
        "In Flander's Fields the poppies grow",
        "between the crosses row on row",
        "that mark our place; and in the sky",
        "the larks, still bravely singing, fly",
        "Scarce heard amidst the guns below",
        "",
        "We are the dead. Short days ago we lived, felt dawn, and saw the sunset glow",
        "loved, and were loved, and now we lie",
        "In Flander's Fields",
        "",
        "Take up our quarrel with the foe;",
        "to you from failing hands we throw the torch",
        "Be yours to hold it high.",
        "if ye break with us who die",
        "We shall not sleep.",
        "Though poppies grow in Flander's fields",
    }
    for _, line in lines do
        print(line)
        time.wait(0.085)
    end
    ]]
    local poem_path = fs.path.join(script:parent(), "poem.luau")
    fs.writefile(poem_path, src)
    local child = process.spawn {
        program = env.executable_path,
        args = { poem_path },
        stream = {
            stdout_capacity = stdout_capacity,
            stderr_capacity = stderr_capacity,
            stdout_truncate = stdout_truncate,
        }
    }
    return child, poem_path
end

local function makesurelinesdoesntbreak()
    local child = spawn_long_waiter()

    for line in child.stdout:lines() do
        print(line)
        break
    end

    print("after possible sigabrt???")
    print(child.stdout:read(nil, 0.12))
end

makesurelinesdoesntbreak()


local function basicread()
    local child = spawn_long_waiter()
    local text = child.stdout:read()
    assert(text == "hehe\n", "text not match hehe\\n")
    local content = child.stderr:read(nil, 0.5)
    child:kill()
    print(content)
end

basicread()

local function readtoofast()
    local child = spawn_long_waiter()
    local text = child.stdout:read(nil, 0)
    assert(text == nil, "we should've read too fast because we're nonblocking")
    child:kill()
end

readtoofast()

local function readupto()
    local child, poem_path = spawn_poem(128, 64, "front")
    local text = child.stdout:read(24, 0.25)
    assert(text ~= nil, "poem text shouldn't be nil in readupto")
    local len = child.stdout:len()
    assert(len > 0, "after reading up to 24 bytes from poem there should be more than zero remaining bytes in the stream")
    assert(#text <= 24, "#text should be <= 24 when readupto 24")
    child:kill()
    fs.file.try_remove(poem_path)
end

readupto()

local function readloop()
    local child = spawn_longrunning()
    local result: { string } = {}
    while time.wait(0.025) do
        local text = child.stdout:read(nil, 0)
        if text then
            table.insert(result, text)
            if str.endswith(text, "Stopping\n") then
                break
            end
        end
    end
    local result = table.concat(result, "\n")
    assert(str.endswith(result, "Stopping\n"))
end

readloop()

local function mixingreads()
    local child = spawn_long_waiter()
    local stdout = child.stdout
    local first = stdout:read_exact(1, 0.5)
    assert(first == "h", "first should be h")
    local readnormally = stdout:read()
    assert(readnormally == "ehe\n")
    child:kill()	
end

mixingreads()

local function readexactbasic()
    local child, poem_path = spawn_poem(222, 32, "back")
    local t0 = os.clock()
    local first_paragraphish = child.stdout:read_exact(176, 0.9)
    local t1 = os.clock() - t0
    assert(t1 > 0.3, "should've taken a bit more than 0.3 seconds to print the first paragraph out")
    assert(first_paragraphish and str.endswith(first_paragraphish, "guns below"), `should end with guns below, got: {first_paragraphish}`)
    assert(#first_paragraphish == 176, "reading exactly 176 bytes should result in 176 length string")
    child:kill()
    fs.file.try_remove(poem_path)
end

cheese.retry(readexactbasic, 3)
