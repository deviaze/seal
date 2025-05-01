local fs = require("@std/fs")

local datadir = fs.dir.from("./tests/data")

local old_movetrees = datadir:find("movetrees"):try_file()
if old_movetrees then
	old_movetrees:remove()
end

datadir:add_tree("movetrees", fs.tree()
	:with_tree("start", fs.tree()
		:with_file("move1", "moveme")
		:with_tree("movefolder", fs.tree()
			:with_file("hi", "hi")
		)
	)
	:with_tree("finish", fs.tree())
)

fs.move(
	"./tests/data/movetrees/start/move1", 
	"./tests/data/movetrees/finish/move1"
)
assert(
	datadir
		:expect_dir("movetrees")
		:expect_dir("start")
		:find("move1"):try_file() == nil,
	"was move1 not moved successfully?"
)

local succ, err = pcall(function()
	fs.move(
		"./tests/data/movetrees/movefolder",
		"./tests/data/movetrees/finish/movefolder"
	)
	return nil
end)
assert(succ == false, "this shouldn't succeed")
assert(tostring(err):match("unable to move"), "should be unable to move bc wrong path typed in")

fs.move(
	"./tests/data/movetrees/start/movefolder",
	"./tests/data/movetrees/finish/movefolder"
)

datadir
	:expect_dir("movetrees")
	:expect_dir("finish")
	:expect_dir("movefolder")

datadir:expect_dir("movetrees"):remove()