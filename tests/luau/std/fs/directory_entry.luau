local fs = require("@std/fs")

local data_path = fs.path.join(".", "tests", "data")

local data_dir = fs.dir.from(data_path)

local function addingandchaining()
	local dirents = data_dir:add_tree("dirents", fs.tree()
		:with_dir("hi", fs.tree()
			:with_file("veryentry.txt", "hi")
			:with_file("init.luau", "return function() end")
		)
	):expect_dir("dirents") -- :add_tree returns self for chaining, not the dir just created

	dirents -- test chaining
		:add_file("etc", "etsy")
		:add_file("starf", "seal")
		-- funny if we can add nested files
		:add_file("hi/cat.meow", "doesthismeow")

	assert(fs.path.exists("./tests/data/dirents/hi/cat.meow"), "why not exist?")
	dirents:remove()
end

addingandchaining()

local function listingdirectory()
	local srcdir = fs.dir.from("./src")
	local srclist = srcdir:list(true)
	local expected_paths = { "./src/main.rs", "./src/std_fs/mod.rs" }
	for _, path in expected_paths do
		assert(table.find(srclist, path), `missing expected path {path}`)	
	end
end

listingdirectory()

local function expecting()
	-- expecting a dir cant result in file lol
	local res, err = pcall(function()
		data_dir:expect_dir("cats.json")
	end)
	assert(res == false, "expecting dir on a file can't succeed?")
	assert(tostring(err):match("expected directory doesn't exist"), "unexpected error message")

	local server_views = data_dir:expect_dir("server-views")
	assert(server_views.name == "server-views", "huh")
end

expecting()

local function dirmetadata()
	local metadata = data_dir:metadata()
	assert(metadata.created_at ~= nil, "where created at?")
end

dirmetadata()