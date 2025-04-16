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

local entry = fs.find("./Cargo.toml"):try_file()
assert(entry ~= nil and entry.type == "File", "Cargo.toml not found?")

local src = fs.find("./src"):try_dir()
assert(src ~= nil, "where src?")
assert(src.name == "src")
assert(src.type == "Directory")

assert(fs.find("hi"):try_dir() == nil, "why do we have a dir named hi")

-- see metadata
local dt = require("@std/time/datetime")

local function meta()
	local mainrs = fs.file("./src/main.rs")
	if mainrs then
		local metadata = mainrs:metadata()
		local now = dt.now().unix_timestamp
		local accessed_at = metadata.accessed_at
		assert(accessed_at ~= nil, "where is accessed at at?")
	end
end

meta()