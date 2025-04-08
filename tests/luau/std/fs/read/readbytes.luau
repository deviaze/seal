--!nonstrict
--!nolint LocalShadow
local fs = require("@std/fs")

local funny_story = "./tests/data/funny_story.txt"

local function getanewbuffer()
	local buffy = fs.readbytes(funny_story)
	assert(type(buffy) == "buffer", "where buffy?")
end

getanewbuffer()

local function getnewbuffercount()
	local buffy = fs.readbytes(funny_story, 0, 27)
	assert(buffer.tostring(buffy) == "Once upon a time in a pixel", "somehow count is off?")
end

getnewbuffercount()

local function readfileintoexistingbuffer()
	-- read first 40 bytes of file into existing buffer
	local buffy = buffer.create(42)
	local resbuffy = fs.readbytes(funny_story, 0, 40, buffy)
	assert(buffy == resbuffy, "not the same buffer? huh")
end

readfileintoexistingbuffer()
