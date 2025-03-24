--!nonstrict
--!nolint LocalShadow
local fs = require("@std/fs")

local funny_story = "./tests/data/funny_story.txt"

local count = 1024
local buffy = buffer.create(count)
fs.readbytes(funny_story, buffy, 0, 0, 40)
assert(buffer.tostring(buffy):match("Once upon a time in a pixelated paradise"), "buffer doesnt match")
fs.readbytes(funny_story, buffy, 10, 60, 10)
assert(buffer.tostring(buffy):match("called"), "overwriting buffer didn't work?")
