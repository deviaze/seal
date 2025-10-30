local fs = require("@std/fs")
local env = require("@std/env")
local process = require("@std/process")
local time = require("@std/time")
local cheese = require("@tests/cheese")

local function runsealwithprompt(prompt: string)
    local src = string.gsub([[
    local input = require("@std/io/input")
    local result = input.readline("PROMPT")
    if typeof(result) == "string" then
        print(result)
    else
        print(`{typeof(result)}: {tostring(result)}`)
    end
    ]], "PROMPT", prompt)
    local dir = fs.dir.ensure(fs.path.join(fs.path.home(), "Downloads"))
    local path = dir:join("promptsrc.luau")
    fs.writefile(path, src)
    local child = process.spawn {
        program = env.executable_path,
        args = { path }
    }
    time.wait(0.125)
    fs.file.try_remove(path)
    return child
end

local function rawlinefallback()
    local child = runsealwithprompt("how are you?: ")
    
    while time.wait(0.12) and child:alive() do
        local res = child.stdout:read(nil, 0)
        if res and res == "how are you?: " then
            child.stdin:write("good!!\n")
        elseif res and res == "good!!" then
            return
        end
        local err = child.stderr:read(nil, 0)
        if err then
            warn(`stderr: {err}`)
        end
    end

    print("child died")
    local good = child.stdout:read()
    assert(good and good:match("good!!"), `should be good here? got: {good}`)
end

cheese.retry(rawlinefallback, 2)