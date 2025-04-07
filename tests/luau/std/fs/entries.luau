--!nolint LocalShadow
local fs = require("@std/fs")

for path, entry in fs.entries(".") do
	if entry.type == "File" then
		assert(typeof(entry.name) == "string", "where entry.name?")
		local content = (entry :: fs.FileEntry):read()
		assert(typeof(content) == "string", "file content is not string?")
	else
		for nested_path, nested_entry in entry:entries() do
		end
	end
end

local entry = fs.find("./Cargo.toml").file
assert(entry ~= nil and entry.type == "File", "Cargo.toml not found?")

local src = fs.find("./src").dir
assert(src ~= nil, "where src?")
assert(src.name == "src")
assert(src.type == "Directory")

assert(fs.find("hi").dir == nil, "why do we have a dir named hi")

local function findfileindir()
	local server_views: fs.DirectoryEntry? = fs.dir("./tests/data/server-views")
	if server_views then
		local index_html = server_views:find("index.html").file
		assert(index_html ~= nil, "can't find index.html")
	end
end

findfileindir()

local function finddirindir()
	local data_folder = fs.find("./tests/data"):unwrap_dir()
	local server_views = data_folder:find("server-views").dir
	assert(server_views ~= nil, "where is server-views?")
end

finddirindir()

-- retry_file/dir changes entry
local function retryfile()
	local nofileyet = "./tests/data/nofileyet.txt"
	local result = fs.find(nofileyet)
	assert(result:exists() == false, "nofileyet actually fileyet")

	fs.writefile(nofileyet, "fileyet")
	assert(result:exists() == true, "what why not fileyet???")

	assert(result.file == nil, "we shouldnt have .file until retry")
	local f = result:retry_file()
	assert(f ~= nil, "retry_file didnt return?")
	assert(result.file == f, "result.file and f aren't identical?")

	fs.removefile(nofileyet)

	local result = result :: fs.FindResult

	assert(result:exists() == false, "shouldn't exist anymore")
	result:retry_file()
	assert(result.file == nil, ".file should be nil again")
end

retryfile()

-- see metadata
local dt = require("@std/time/datetime")

local function meta()
	local mainrs = fs.file("./src/main.rs")
	if mainrs then
		local metadata = mainrs:metadata()
		local now = dt.now().unix_timestamp
		local accessed_at = metadata.accessed_at
		if accessed_at then
			print(accessed_at:format_local(dt.common_formats["MM/DD/YYYY HH:MM (AM/PM)"]))
			assert(now - accessed_at.unix_timestamp < 100, "time is off?")
		end
	end
end

meta()