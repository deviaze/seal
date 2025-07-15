local process = require("@std/process")
local env = require("@std/env")
local fs = require("@std/fs")
local str = require("@std/str")

local seal_path = env.executable_path
local longrunning_path = fs.path.join(script:parent(), "longrunning.luau")

local function spawn_longrunning(): process.ChildProcess
    local child = process.spawn {
        program = seal_path,
        args = { longrunning_path, "85", "50" },
    }
    return child
end

local function spawn_poem(): (process.ChildProcess, string)
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

    fs.removefile(script:path())
    ]]
    local poem_path = fs.path.join(script:parent(), "poem.luau")
    fs.writefile(poem_path, src)
    local child = process.spawn {
        program = env.executable_path,
        args = { poem_path }
    }
    return child, poem_path
end

local function intobuffer()
    local child = spawn_longrunning()
    local buffy = buffer.create(1024)
    local offset = 0
    while child:alive() do
        if child.stdout:size() > 0 then
            local count = child.stdout:readbytes(buffy, offset)
            if count then
                offset += count
            end
        end
    end
    local tostringed = buffer.tostring(buffy)
    assert(str.startswith(tostringed, "Starting..."), "buffer.tostring(buffy) should start with Starting...")
    assert(str.endswith(tostringed, "\0\0\0\0", "buffer.tostring(buffy) should have a bunch of \\0 bytes behind it because of buffer.tostring reasons"))
    assert(string.match(tostringed, "\nStopping\n"), "buffer.tostring(buffy) should contain \\nStopping\\n but not at the end")
end

intobuffer()

local function cantfitintowholebuffer()
    local child, poem_path = spawn_poem()
    local buffy = buffer.create(32)
    local s, result = pcall(function(): nil
        local offset = 0
        while child:alive() do
            local success, remaining = child.stdout:readbytes_exact(12, buffy, offset)
            if success then
                offset += 12
            end
        end
        return nil
    end)
    assert(typeof(result) == "error" and tostring(result):match("can't fit offset"), "we should get an error for invalid readbytes_exact usage")
    fs.file.try_remove(poem_path)
end

cantfitintowholebuffer()

local function canreadintowholebufferwithchecks()
    local child, poem_path = spawn_poem()
    local buffy = buffer.create(32)
    while child:alive() do
        if child.stdout:size() >= 32 then
            -- print("child is big enough")
            child.stdout:readbytes_exact(32, buffy)
            break
        end
    end
    assert(buffer.tostring(buffy) == "In Flander's Fields the poppies ", "Exactly 32 bytes should've been read")
    fs.file.try_remove(poem_path)
end

canreadintowholebufferwithchecks()

local function vibereadingintobuffer()
    local child, poem_path = spawn_poem()
    local buffy = buffer.create(512)
    local offset = 0
    while child:alive() do
        local count = child.stdout:readbytes(buffy, offset)
        if count then
            offset += count
        end
    end
    assert(buffer.tostring(buffy):match("Take up our quarrel"), "should've read the entire thing into 512 size buffer")
    fs.file.try_remove(poem_path)
end

vibereadingintobuffer()

local function vibereadingintosmallbuffer()
    -- someone forgot an explicit offset < length check!
    local child, poem_path = spawn_poem()
    local s, result = pcall(function(): nil
        local buffy = buffer.create(64)
        local offset = 0
        while child:alive() do
            local count = child.stdout:readbytes(buffy, offset)
            if count then
                offset += count
            end
        end
        return nil
    end)
    fs.file.try_remove(poem_path)
    assert(s == false, "reading into small buffer without explicit overflow check shouldn't work")
    assert(
        typeof(result) == "error" and tostring(result):match("add an explicit"),
        "we should get informative buffer overflow error message"
    )
end

vibereadingintosmallbuffer()

local function correctlyreadingintosmallbuffershouldwork()
    local child, poem_path = spawn_poem()
    local offset = 0
    local buffer_size = 85
    local buffy = buffer.create(buffer_size)
    while child:alive() and offset < buffer_size do
        local count = child.stdout:readbytes(buffy, offset)
        if count then
            offset += count
        end
    end
    fs.file.try_remove(poem_path)
    assert(
        str.endswith(buffer.tostring(buffy), "\nthat mark our pla"),
        "correctly reading into the small buffer should result in endswith \"\\nthat mark our pla\""
    )
end

correctlyreadingintosmallbuffershouldwork()
