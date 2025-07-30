local fs = require("@std/fs")
local path = fs.path

local datadir = fs.dir.from(path.join(".", "tests", "data"))

local testentrypath = path.join(datadir.path, "testentry.txt")
if path.exists(testentrypath) then
	fs.removefile(testentrypath)
end

local function canwriteentry()
	local file = fs.file.create(testentrypath)
	local s, f = pcall(function()
		fs.file.create(testentrypath)
		return nil
	end)
	assert(s == false, "shouldnt be able to create file that alr exists??")
end

canwriteentry()

local function appendfile()
	-- append to file
	local appendfilepath = path.join(datadir.path, "appendfile.txt")
	fs.writefile(appendfilepath, "startercontent")
	local appendfile = fs.find(appendfilepath):unwrap_file()

	assert(appendfile:read() == "startercontent", "wrong content in appendfile")
	appendfile:append("andmore")
	assert(appendfile:read() == "startercontentandmore", "appendfile:append didn't work?")

	fs.removefile(appendfilepath)
end

appendfile()

local function canmoveto()
	local dir1 = fs.dir.ensure(datadir:join("movedir1"))
	local dir2 = fs.dir.ensure(datadir:join("movedir2"))
	local file = dir1
		:add_file("moveme.txt", "meow")
		:expect_file("moveme.txt")
	file:move_to(dir2:join("moveme.txt"))
	assert(fs.path.exists(dir2:join("moveme.txt")), "moveme.txt not moved?")
	dir1:remove()
	dir2:remove()
end

canmoveto()