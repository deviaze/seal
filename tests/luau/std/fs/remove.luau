--!nonstrict
-- local fs = require("@std/fs")

-- fs.writefile { 
-- 	path = "./meow.txt",
-- 	content = "meowewnmweo2",
-- }

-- print(fs.readfile("./meow.txt"))

-- fs.remove({ file = "./meow.txt" })
-- -- fs.remove {
-- -- 	file = "./meow.txt"
-- -- }
local fs = {} :: { remove: (path: string, options: { 
	recurse: boolean?, force: boolean?, remove_absolute_path: boolean? 
}) -> () }

fs.remove("./meow.txt", {
	recurse = true,
	force = true,
	remove_absolute_path = true,
})

-- fs.remove { dir = "./meow" }

fs.removedir("./meow")

fs.remove {
	directory = "./meow",
	remove_absolute_path = false,
}

fs.removefile("meow.txt")

fs.remove("./meow", true)

fs.readfile("")
fs.removetree("./meow")

fs.writetree("./meow", {
	files = {
		{ name = "meow.txt", content = "meow" },
		{ name = "main.luau", content = 'print("hello world")'},
	}
})