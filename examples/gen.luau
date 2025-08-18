-- the script that started ./docs/standard-library/**

local fs = require("@std/fs")
local str = require("@std/str")
local tt = require("@extra/tt")

local std_dir = fs.dir.from(fs.path.cwd())
    :expect_dir(".seal")
    :expect_dir("typedefs")
    :expect_dir("std")

local standard_libraries = std_dir:list(true, function(path: string)
    return str.endswith(path, ".luau")
end)

standard_libraries = tt.arraymap(standard_libraries, function(_, path: string): string?
    local child = fs.path.child(path) :: string
    if not str.startswith(child, "_") and child ~= "init.luau" then
        local back = string.gsub(path, `{std_dir.path}.`, "")
        return str.trimback(back, ".luau")
    else
        return nil
    end
end)

for _, lib in standard_libraries do
    local relative_path = fs.path.join(script:parent(), `{lib}.md`)
    local parent_must_exist_path = fs.path.parent(relative_path) :: string
    fs.makedir(parent_must_exist_path, { create_missing = true, error_if_exists = false })
    fs.writefile(relative_path, `# {lib}\n`)
end
