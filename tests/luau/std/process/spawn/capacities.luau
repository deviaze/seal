local fs = require("@std/fs")
local env = require("@std/env")
local process = require("@std/process")
local time = require("@std/time")
local str = require("@std/str")

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

local function restrictedcapacity()
    local child, poem_path = spawn_poem(52, 42, "front")
    assert(child.stdout:capacity() == 52, "stdout capacity should be 52")
    assert(child.stderr:capacity() == 42, "stderr capacity should be 42")
    time.wait(1)
    local res = child.stdout:read()
    assert(typeof(res) == "string", "reading from stdout here should've worked")
    assert(not str.startswith(res, "In Fla"), "waiting for 1 second should've overfilled buffer so \"In Fla\" should've already been drained")
    fs.file.try_remove(poem_path)
end

restrictedcapacity()

local function truncatebackactuallytruncatesback()
    local child, poem_path = spawn_poem(62, 0, "back")
    time.wait(0.6)
    local current_size = child.stdout:len()
    assert(current_size == 62, `inner should be fully full by now, got: {current_size}`)
    local text = child.stdout:read()
    assert(typeof(text) == "string", "text should be a string here")
    assert(#text == 62, "stream was 62 bytes but read string not?")
    assert(str.startswith(text, "In Flander's"), "the front shouldn't be truncated if stdout_truncate == \"back\"")
    fs.file.try_remove(poem_path)
end

truncatebackactuallytruncatesback()