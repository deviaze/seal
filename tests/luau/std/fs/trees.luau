--!strict

local fs = require("@std/fs")
local datadir = "./tests/data"

local writetreesdir = fs.path.join(datadir, "writetrees")

if fs.dir(writetreesdir) then
    fs.removetree(writetreesdir)
end

fs.writetree(writetreesdir, fs.tree()
    :with_file("name", "content")
    :with_file("main.luau", 'return "hi"')
    :with_tree("mymodule", fs.tree()
        :with_file("init.luau", 'return "mymodule"')
        :with_tree("meow", fs.tree()
            :with_file("child.json", "{}")
        )
    )
)

local path = require("@std/fs/path")

assert(path.exists(writetreesdir), "where top level dir?")
assert(path.exists(path.join(writetreesdir, "name")), "where name?")
local mymoduledir = path.join(writetreesdir, "mymodule")
assert(path.exists(mymoduledir), "where mymodule?")
assert(path.exists(path.join(mymoduledir, "init.luau")), "where mymodule/init.luau")
assert(path.exists(path.join(mymoduledir, "meow")), "where mymodule/meow/")
assert(path.exists(path.join(mymoduledir, "meow", "child.json")), "where mymodule/meow/child.json")

fs.removetree(writetreesdir)

local typedefs = fs.readtree("./.seal/typedefs")
local typedefs_clone_dir = path.join(datadir, ".typedefs")
if fs.find(typedefs_clone_dir):exists() then
    fs.removetree(typedefs_clone_dir)
end

fs.writetree(typedefs_clone_dir, typedefs)
assert(path.exists(path.join(typedefs_clone_dir, "std", "fs")), "expected datadir/.typedefs/std/fs to exist")
fs.removetree(typedefs_clone_dir)