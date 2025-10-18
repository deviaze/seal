local fs = require("@std/fs")
local io = require("@std/io")
local str = require("@std/str")
local env = require("@std/env")
local colors = require("@std/colors")
local process = require("@std/process")

local tt = require("@extra/tt")

type FailedTest = {
    path: string,
    err: string,
}

type IgnoredFunctionByName = {
    path: string,
    reason: string,
}

type IgnoredFunctionByPath = {
    function_name: string,
    reason: string,
}

type IgnoredFunctionsByName = {
    [string]: IgnoredFunctionByName
}

type IgnoredFunctionsByPath = {
    [string]: { IgnoredFunctionByPath },
}

type Tests = {
    queue: { string },
    passed: { string },
    failed: { FailedTest },
    invalid: { [string]: string },
    ignored_functions_by_name: IgnoredFunctionsByName,
    ignored_functions_by_path: IgnoredFunctionsByPath,
}

local function new_test_queue(): { string }
    local ignore_files = {
        "./tests/luau/std/net/server/serve.luau", -- manual test
        -- the below are not test files but are required by/imported by other tests
        "./tests/luau/std/thread/get-threads/send_request.luau",
        "./tests/luau/std/net/server/client.luau",
        "./tests/luau/std/io/raw.luau", -- rawmode messes with everything
        "./tests/luau/std/io/events.luau", -- rawmode messes with everything
        "./tests/luau/std/thread/conc_1.luau",
        "./tests/luau/errors/another_module.luau",
        "./tests/luau/std/process/spawn/ception.luau",
        "./tests/luau/globals/require/basic-requires/relative_data.luau",
    }
    local file_list = fs.listdir(fs.path.join(".", "tests", "luau"), true, function(path: string)
        return 
            str.endswith(path, ".luau") 
            and not table.find(ignore_files, path)
            and not str.startswith(path, "./tests/luau/std/io/prompts") -- all of these require rawmode or smth
    end)
    table.insert(file_list, "./tests/scripts/seal_setup_test.luau")
    return file_list
end

local function get_ignored_functions(test_files: { string }): (IgnoredFunctionsByName, IgnoredFunctionsByPath)
    local ignored_by_name: IgnoredFunctionsByName = {}
    local ignored_by_path: IgnoredFunctionsByPath = {}

    for _, test_path in test_files do
        local ignored_in_this_file: { [string]: true } = {}
        local ignored_by_path_in_this_file: { IgnoredFunctionByPath } = {}

        local last_reason: string = ""
        local grab_next_line = false
        local looking_for_asserts_in_function: string? = nil

        for n, line in fs.readlines(test_path) do
            if grab_next_line then
                local function_name = string.match(line, "%s*local function ([%w_]+)%(")
                if function_name then
                    ignored_in_this_file[function_name] = true
                    ignored_by_name[function_name] = {
                        path = test_path,
                        reason = last_reason,
                    }
                    table.insert(ignored_by_path_in_this_file, {
                        function_name = function_name,
                        reason = last_reason,
                    })
                    last_reason = ""
                    grab_next_line = false
                    looking_for_asserts_in_function = function_name
                end
            elseif string.match(line, "%s*--/sealignore") then
                local found_reason = string.match(line, "%s*--/sealignore[:]*%s*([%w%s%p]+)$")
                if found_reason then
                    last_reason = found_reason
                    grab_next_line = true
                    continue
                end
            elseif looking_for_asserts_in_function then
                if string.match(line, `{looking_for_asserts_in_function}%(`) then
                    looking_for_asserts_in_function = nil
                elseif string.match(line, "^%s*assert%(") then
                    error(`assert on line {n} in ignored function '{looking_for_asserts_in_function}' must be commented out!`)
                end
            end
        end

        if #ignored_by_path_in_this_file > 0 then
            ignored_by_path[test_path] = ignored_by_path_in_this_file
        end
    end

    return ignored_by_name, ignored_by_path
end

local test_files = new_test_queue()
local ignored_by_name, ignored_by_path = get_ignored_functions(test_files)
local tests: Tests = {
    queue = test_files,
    passed = {},
    failed = {},
    invalid = {},
    ignored_functions_by_name = ignored_by_name,
    ignored_functions_by_path = ignored_by_path,
}

local test_pool: { [string]: process.ChildProcess } = {} do
    for _, test_path in tests.queue do
        local child = process.spawn {
            program = env.executable_path, -- usually ./target/debug/seal or ./target/release/seal
            args = { test_path }
        }
        test_pool[test_path] = child
    end
end

local start_time = os.clock()
local tests_with_stdin = {"io/init", "stdin", "input", "prompt"}

while (os.clock() - start_time) < 20 do
    for test_path, child in test_pool do
        -- wait til child dies
        if not child:alive() then
            -- check stderr for any error messages; anything written to stderr = fail
            if 
                child.stderr:len() > 0
                and fs.path.child(test_path) ~= "err.luau" -- err.luau should write to stderr
            then
                local err = child.stderr:read()
                if err and err:match("ERR") or err and err:match("PANIC") or err and err:match("WARN") then
                    print(` ❌ {colors.red("test failed")}: {test_path}`)
                    table.insert(tests.failed, {
                        path = test_path,
                        err = err,
                    })
                end
            else
                table.insert(tests.passed, test_path)
                print(` {if #tests.passed < 10 then " " else ""}{#tests.passed} {colors.yellow("test successful")}: {test_path}`)
                -- check if this file has any ignored functions and warn about them
                local ignored_in_this_file = tests.ignored_functions_by_path[test_path]
                if ignored_in_this_file then
                    for function_name, info in ignored_in_this_file do
                        print(`    ⚠️  test case '{info.function_name}' ignored due to reason: {info.reason}`)
                    end
                end
            end

            test_pool[test_path] = nil
        else
            -- some tests need user input
            for _, p in tests_with_stdin do
                if test_path:match(p) then
                    if str.endswith(test_path, "simple_prompts.luau") then
                        child.stdin:write("y\n")
                    else
                        child.stdin:write(`{math.random(1, 223)}\n`)
                    end
                end
            end
        end
    end

    if #tt.keys(test_pool :: any) == 0 then
        break
    end
end

for test_path, child in test_pool do
    if child:alive() then
        print(`{(colors.magenta("time limit exhausted"))}: {test_path}`)
        child:kill()
        table.insert(tests.failed, {
            path = test_path,
            err = "time limit exhausted",
        })
    end
end

print("")

local elapsed = math.round((os.clock() - start_time) * 100) / 100

print(`elapsed: {colors.blue(`{elapsed} seconds`)}`)
print(`test files: {#test_files}`)
print(`{colors.yellow("passed")}: {#tests.passed}`)
print(`{colors.red("failed")}: {#tests.failed}`)

if #tests.failed > 0 then
    print("")
    for index, test in tests.failed do
        print(`{colors.red("[FAILED]")} #{index} {test.path} \n{test.err}\n`)
    end
    print("")
    io.output.ewrite(colors.red("[FAILED]")) -- make sure we fail cicd
end
