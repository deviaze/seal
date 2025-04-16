--!nolint LocalShadow

local fs = require("@std/fs")
local mlua = require("@interop/mlua")

local function findfile()
	local cargo_toml_result: fs.FindResult = fs.find("./Cargo.toml")
	assert(cargo_toml_result.ok == true :: boolean, "Cargo.toml result.ok should be true")
	assert(cargo_toml_result.type == "File", "why is Cargo.toml result.type not a file??")
	local cargo_toml = cargo_toml_result:try_file()
	assert(cargo_toml ~= nil, "where is cargo_toml")
	assert(cargo_toml.type == "File", "why is cargo_toml not file")
end

findfile()

local function finddir()
	local src_result: fs.FindResult = fs.find("./src")
	assert(src_result.ok == true :: boolean, "why is src result not ok?")
	assert(src_result:exists() == true, "why does src result exist not exist?")
	assert(src_result.type == "Directory", "why is src result not a directory?")
	local src_entry = src_result:try_dir()
	assert(src_entry ~= nil, "src entry nil?")
	assert(src_entry.type == "Directory", "why is src_entry not directory?")
end

finddir()

local function findfileindir()
	local server_views: fs.DirectoryEntry? = fs.dir("./tests/data/server-views")
	if server_views then
		local index_html = server_views:find("index.html"):try_file()
		assert(index_html ~= nil, "can't find index.html")
	end
end

findfileindir()

local function finddirindir()
	local data_folder = fs.find("./tests/data"):unwrap_dir()
	local server_views = data_folder:find("server-views"):try_dir()
	assert(server_views ~= nil, "where is server-views?")
end

finddirindir()

-- try_file/dir changes entry
local function retryfile()
	local nofileyet = "./tests/data/nofileyet.txt"
	local result = fs.find(nofileyet)
	assert(result:exists() == false, "nofileyet actually fileyet")

	fs.writefile(nofileyet, "fileyet")
	assert(result:exists() == true, "what why not fileyet???")

	assert(result:try_file() ~= nil, "we should have file now")
	assert(result.ok == true, "after we make file result.ok should be true")

	fs.removefile(nofileyet)

	assert(result:exists() == false, "shouldn't exist anymore")
	assert(result:try_file() == nil, "try_file should be nil again")
end

retryfile()

local function unwrap()
	local tests_dir = fs.find("./tests"):unwrap_dir()
	assert(tests_dir ~= nil, "dir huh")
	local main_rs = fs.find(fs.path.join(".", "src", "main.rs")):unwrap_file()
	assert(main_rs ~= nil, "file huh")
end

unwrap()

local function unwraperrors()
	local s, err = pcall(function()
		local invalid_dir = fs.find("invaliddir"):unwrap_dir()
	end)
	assert(s == false and mlua.iserror(err), "unwrapping invalid_dir should've errored")
	assert(tostring(err):match("nonexistent directory"), "bad error message")
	local s, err = pcall(function()
		local invalid_file = fs.find("./hmm.rs"):unwrap_file()
	end)
	assert(s == false and mlua.iserror(err), "unwrapping invalid_file should've errored")
	assert(tostring(err):match("nonexistent file"), "bad error message")
end

unwraperrors()

local function fsfile__call()
	local src = fs.file(fs.path.join(".", "src"))
	assert(src == nil, "src is not a file")
	local main_rs = fs.file(fs.path.join(".", "src", "main.rs"))
	assert(main_rs ~= nil, "why is main.rs nil from fs.file call")
end

fsfile__call()

local function fsdir__call()
	local src = fs.dir("./src")
	assert(src ~= nil, "where src")
	local main_rs = src:find("main.rs"):unwrap_file()
end

fsdir__call()
