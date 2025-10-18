local fs = require("@std/fs")
local env = require("@std/env")
local process = require("@std/process")
local cheese = require("@tests/cheese")

local function watchception()
    local ception_path = fs.path.join(script:parent(), "ception.luau")
    
    local child = process.spawn {
        program = env.executable_path,
        args = { ception_path }
    }

    local t0 = os.clock()
    local not_first = child.stdout:read(nil, 0.0)
    assert(
        not_first == nil, 
        "not_first should be nil here since :read with timeout 0.0 should be nonblocking"
    )
    local first = child.stdout:read(5, 0.2)
    assert(typeof(first) == "string" and first == "first", "first should be first here")
    local newline = child.stdout:read()
    assert(typeof(newline) == "string" and newline == "\n", "a newline should be printed after first")
    local o_o = child.stdout:read(nil, 0.8)
    assert(
        typeof(o_o) == "string"
        and (o_o:match("o_o")), 
        `o_o should be found now because we've waited just long enough after it should've been printed (got: {o_o})`
    )
    
    local count = 1
    for message in child.stdout:lines() do
        if count == 1 then
            assert(message == "got cats", "should be got cats here")
        elseif count == 2 then
            assert(message == "got dogs", "should be got dogs here")
        elseif count == 3 then
            assert(message == "got seals", "should be got seals here")
        end
        count += 1
    end    
end

cheese.retry(watchception, 2)

