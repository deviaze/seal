--!strict

local fs = require("@std/fs")
local datadir = "./tests/data"

local writetreesdir = fs.path.join(datadir, "writetrees")

if fs.dir(writetreesdir) then
    fs.removetree(writetreesdir)
end

fs.writetree(writetreesdir, {
    fs.file.build("name", "content"),
    fs.file.build("main.luau", "return 'hi'"),
    fs.dir.build("mymodule", {
        fs.file.build("init.luau", "return 'mymodule'"),
        fs.dir.build("meow", {
            fs.file.build("child.json", "{}")
        })
    })
})

local path = require("@std/fs/path")

assert(path.exists(writetreesdir), "where top level dir?")
assert(path.exists(path.join(writetreesdir, "name")), "where name?")
local mymoduledir = path.join(writetreesdir, "mymodule")
assert(path.exists(mymoduledir), "where mymodule?")
assert(path.exists(path.join(mymoduledir, "init.luau")), "where mymodule/init.luau")
assert(path.exists(path.join(mymoduledir, "meow")), "where mymodule/meow/")
assert(path.exists(path.join(mymoduledir, "meow", "child.json")), "where mymodule/meow/child.json")

fs.removetree(writetreesdir)