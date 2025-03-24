--!nolint LocalShadow
local fs = require("@std/fs")

for path, entry in fs.entries("./src") do
	print(path)
	if entry.type == "Directory" then
		print(entry)
	end
end

-- for path, entry in fs.entries(".") do
-- 	if entry.type == "File" then
-- 		assert(entry.extension, "where extension?")
-- 		assert(entry.name ~= nil, "where entry.name?")
-- 		local content = (entry :: fs.FileEntry):read()
-- 		assert(typeof(content) == "string", "file content is not string?")
-- 	else
-- 		for nested_path, nested_entry in entry:entries() do
-- 		end
-- 	end
-- end

-- local entry = fs.find("./Cargo.toml")
-- assert(entry ~= nil and entry.type == "File", "Cargo.toml not found?")

-- local src = fs.find({directory = "./src"})
-- assert(src ~= nil, "no src?")
-- assert(src.name == "src", "src name not src?")
-- assert(src.type == "Directory", "src not a directory???")

-- local ninexists = fs.find { directory = "hi" }
-- assert(ninexists == nil)

-- assert(fs.find{file = "Cargo.toml"} ~= nil)

-- local try = require("@std/testing/try")

-- -- handle cases
-- do
-- 	try(function()
-- 		fs.entries("")
-- 		return nil
-- 	end):expect_err("No such file or directory")
-- end

-- -- append to file
-- local appendfile = fs.create { file = {
-- 	name = "./tests/data/appendfile.txt",
-- 	content = "startercontent"
-- }} :: fs.FileEntry

-- if appendfile then
-- 	local content = (appendfile :: fs.FileEntry):read() :: string
-- 	assert(content == "startercontent", "saving file apparently didnt write 'startercontent'?")
-- 	local appendfile: fs.FileEntry = appendfile
-- 	appendfile:append("andmore")
-- 	assert(appendfile:read() == "startercontentandmore", "appendfile:append didn't work right?")
-- end

-- local function findfile()
-- 	local server_views = fs.dir("./tests/data/server-views")
-- 	if server_views then
-- 		local index_html = server_views:file("index.html")
-- 		assert(index_html ~= nil, "can't find index.html")
-- 		assert(index_html.extension == "html")
-- 	end
-- end

-- findfile()

-- local function finddir()
-- 	local data_folder = fs.dir("./tests/data")
-- 	if data_folder then
-- 		local server_views = data_folder:dir("server-views")
-- 		assert(server_views ~= nil, "cant find server-views")
-- 	end
-- end

-- finddir()