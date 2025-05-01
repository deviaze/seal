local fs = require("@std/fs")

local data_dir = fs.dir.from("./tests/data")

local function basicremovefile()
	data_dir:add_file("removeme", "oops")
	
	fs.removefile(fs.path.join(data_dir.path, "removeme"))
	
	local s, f = pcall(function()
		fs.removefile("startsfeiontnesart.rstas")
		return nil
	end)
	
	assert(s == false, "removing nonexistent file cant succeed")
	assert(tostring(f):match("not found"), "why not not found?")
end

basicremovefile()

-- can we remove a dir with fs.removefile?

local function removefilecantremovedir()
	data_dir:add_tree("removeme", fs.tree()
		:with_file("hi", "im file")
    )
	local s, f = pcall(function()
		fs.removefile(fs.path.join(data_dir.path, "removeme"))
		return nil
	end)
	assert(s == false, "why")
	data_dir:expect_dir("removeme"):remove()
end

removefilecantremovedir()

local function canremovetreeremovefile()
	data_dir:add_file("removemeremovetree", "imactuallyfile")
	local s, f = pcall(function()
		fs.removetree(fs.path.join(data_dir.path, "removemeremovetree"))
		return nil
	end)
	assert(s == false, "removetree shouldnt be able to remove singluar files")
	data_dir:expect_file("removemeremovetree"):remove()
end

canremovetreeremovefile()