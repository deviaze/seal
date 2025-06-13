--!nolint LocalShadow
local fs = require("@std/fs")
local res = fs.readfile("./tests/data/funny_story.txt")
assert(res:match("Luau"), "fs/read/readfile did we read wrong file?")

-- we can read binary file?
local res = fs.readfile("./tests/data/nanuk.png")

-- make sure we get comparable results w/ fs.readbytes
local bytes = fs.readbytes("./tests/data/nanuk.png")
assert(#res == buffer.len(bytes), "fs.readfile and fs.readbytes giving different length?")

local success, result = pcall(function()
	return fs.readfile("./trsastar")
end)

assert(success == false, "fs.readfile invalid file should fail")
print(typeof(result))
assert(typeof(result) == "error" and tostring(result):match("File/directory not found"), "Unexpected error message")