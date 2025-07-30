local fs = require("@std/fs")

local src_path = fs.path.join(".", "src")

local function listbasic()
    local src_files = fs.listdir(src_path)
    assert(table.find(src_files, fs.path.join(".", "src", "main.rs")), "main.rs should be in there")
    assert(table.find(src_files, fs.path.join(".", "src", "std_fs", "find.rs")) == nil, "shouldn't be recursive")
end

listbasic()

local function listrec()
    local src_files = fs.listdir(src_path, true)
    assert(table.find(src_files, fs.path.join(".", "src", "std_fs", "find.rs")), "recursive should find find.rs")
end

listrec()

local function listfilter()
    local src_files = fs.listdir(src_path, true, function(path: string)
        return if string.match(path, "%.luau$") then true else false
    end)
    assert(table.find(src_files, fs.path.join(".", "src", "require", "resolver.luau")), "couldn't find resolver.luau with filter fn?")
end

listfilter()