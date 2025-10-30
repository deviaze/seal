local env = require("@std/env")
local fs = require("@std/fs")
local process = require("@std/process")

local gsw_path = fs.path.join(script:parent(), "gsw.luau")
local lgti_path = fs.path.join(script:parent(), "commands.luau")
local fakeseal_path = fs.path.join(script:parent(), "fakeseal.luau")

local function gsw()
    local result = process.run {
        program = env.executable_path,
        args = { gsw_path, "mybranchy", "--confirm", "--create" }
    }

    assert(result.ok, "running gsw should be okay")
    result = result:unwrap()
    assert(
        result:match("Should allow branch creation"),
        "gsw should allow branch creation"
    )
    assert(
        result:match("Should show confirmation prompt"),
        "gsw should show confirmation prompt"
    )
    assert(
        result:match("Switching to branch mybranchy"),
        "gsw should switch to branch 'mybranchy'"
    )
end

gsw()

local function lgti_add_repo()
    local result = process.run {
        program = env.executable_path,
        args = { lgti_path, "add", "deviaze/seal", "--src=./tests/run.luau", "--folder=./deps", "--rename=sealtesting"}
    }

    assert(result.ok, "running lgti add should be okay")
    result = result:unwrap()
    assert(
        result:match("Got command add"),
        `Should be command add, got {result}`
    )
    assert(
        result:match("Got unparsed repo name deviaze/seal"),
        `Should have gotten unparsed repo name deviaze/seal, got: {result}`
    )
    assert(
        result:match("Got parsed repo name sealtesting"),
        `Should have gotten parsed repo name sealtesting, got: {result}`
    )
    assert(
        result:match("Got folder name ./deps"),
        `Should have gotten folder name ./deps, got: {result}`
    )
end

lgti_add_repo()

local function lgti_a_repo()
    local result = process.shell(
        `{env.executable_path} {lgti_path} a luau-lang/luau`
    )
    assert(result.ok, "lgti command alias a for add should be okay")
    result = result:unwrap()
    assert(
        result:match("Got command add"),
        "Should've got command add even with alias a"
    )
    assert(
        result:match("Got unparsed repo name luau%-lang/luau"),
        `Unparsed repo name should be luau-lang/luau, got: {result}`
    )
    assert(
        result:match("Got folder name ./libraries"),
        `--folder should default to ./libraries, got: {result}`
    )
end

lgti_a_repo()

local function runfakeseal(c: string): string
    return process.shell(`{env.executable_path} {fakeseal_path} {c}`):unwrap()
end

local function fakeseal()
    local result = runfakeseal("myfile.luau")
    assert(
        result == "default filename myfile.luau with args", 
        `expected myfile.luau with no args, got: {result}`
    )
    result = runfakeseal("myfile.luau meow meow2")
    assert(
        result == "default filename myfile.luau with args meow, meow2",
        `expected myfile.luau with args meow, meow2, got: {result}`
    )
end

fakeseal()

local function fakesealrun()
    local result = runfakeseal("run meow1 meow2")
    assert(
        result == "run with args meow1, meow2",
        `expected run with args meow1, meow2, got: {result}`
    )
end

fakesealrun()

local function fakesealsetup()
    local result = runfakeseal("setup p")
    assert(
        result == "codebase style project",
        `should be validated into codebase style project, got: {result}`
    )
    local badvalidation = process.shell(
        `{env.executable_path} {fakeseal_path} setup pr`
    )
    assert(
        badvalidation.stdout:match("invalid codebase style"),
        `should have gotten invalid codebase style, got: {badvalidation.stdout}`
    )
end

fakesealsetup()