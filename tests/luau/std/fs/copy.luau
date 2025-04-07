local fs = require("@std/fs")

local copytreepath = "./tests/data/copytree"
if fs.path.exists(copytreepath) then
	fs.removetree(copytreepath)
end

local copytree = fs.dir.create("./tests/data/copytree")
copytree
	:add_file("regularfile", "regularcontent")
	:add_tree("atree", fs.tree()
		:with_dir("subdir", fs.tree()
			:with_file("hi", "bye")
		)
		:with_file("otherfile", "other")
	)
	:add_tree("target", fs.tree())

fs.copy(
	fs.path.join(copytree.path, "regularfile"),
	fs.path.join(copytree.path, "target")
) -- copying file to a directory should append basename onto directory destination path

assert(
	fs.path.exists(fs.path.join(copytree.path, "target", "regularfile")), 
	"regularfile not copied correctly?"
)

local s, f = pcall(function()
	fs.copy(
		fs.path.join(copytree.path, "atree"),
		fs.path.join(copytree.path, "target")
	)
end)

assert(s == false, "can't copy dir into dir implicitly")
assert(tostring(f):match("Error on path"), "can't implicitly copy dir into dir")

fs.copy(
	fs.path.join(copytree.path, "atree"),
	fs.path.join(copytree.path, "target", "atree")
)

copytree
	:expect_dir("target")
	:expect_dir("atree")
	:expect_dir("subdir")

copytree:remove()