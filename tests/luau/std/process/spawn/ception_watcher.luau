local fs = require("@std/fs")
local env = require("@std/env")
local process = require("@std/process")

local function watchception()
    local ception_path = fs.path.join(script:parent(), "ception.luau")
    
    local child = process.spawn {
        program = env.executable_path,
        args = { ception_path }
    }

    local t0 = os.clock()
    local not_owo = child.stdout:read(0.0)
    assert(
        not_owo == nil, 
        "not_owo should be nil here since :read on empty is nonblocking, just-spawn blocking is explicitly disabled \z
        and because the child thread literally waits 0.75 seconds before writing o_o"
    )
    assert(
        (os.clock() - t0) < 0.7, 
        "if child.stdout:read were blocking this would trigger"
    )
    local o_o = child.stdout:read(0.755)
    assert(o_o == "o_o\n", "o_o should be found now because we've waited just long enough after it should've been printed")
    
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

watchception()
