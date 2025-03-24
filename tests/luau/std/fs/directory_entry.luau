local fs = require("@std/fs")

local src_dir = fs.dir.from("./src")
print(src_dir)
print(src_dir:list(true))