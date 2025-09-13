local fs = require("@std/fs")
local env = require("@std/env")

local function ensuredir()
	local src = fs.dir.ensure("./src")
	assert(src.path == "./src", "why not ensure src?")
	assert(#src:list() > 0, "oh no we just wiped the whole srcdir??")

	local previously_nonexistent_path = fs.path.join(".", "tests", "data", "meow")
	local previously_nonexistent_dir = fs.dir.ensure(previously_nonexistent_path)
	assert(fs.path.exists(previously_nonexistent_path), "previously nonexistent should exist now?")
	assert(previously_nonexistent_dir.path == previously_nonexistent_path, "what's wrong with path?")
	previously_nonexistent_dir:remove()
end

ensuredir()

local permission_denied_path = 
	if env.os == "Windows" then [[C:\System]]
	elseif env.os == "Linux" then "/root"
	elseif env.os == "Android" then "/"
	elseif env.os == "MacOS" then "/private/var/root"
	else "/root"

local data_dir = fs.dir.from("./tests/data")

local function tryremove()
	local dir_to_remove = data_dir:add_tree(
		"dirtoremove", fs.tree()
			:with_file("some", "things")
	):expect_dir("dirtoremove")

	local success, result = fs.dir.try_remove(dir_to_remove.path)
	assert(success == true and result == "Ok", "cant try remove dir?")
end

tryremove()

if env.os ~= "Windows" and env.os ~= "Android" then -- returns NotFound on Windows instead of PermissionDenied and I don't want to test removing a REAL important path
	local function tryremovepermissiondenied()
		local denied_path = fs.path.join(permission_denied_path, "hi")
		local success, result, kind = fs.dir.try_remove(denied_path)
		assert(success == false and result == "PermissionDenied", "removing permisison denied dir should be permission denied")
	end
	
	tryremovepermissiondenied()
end

local function tryremovenotfound()
	local not_found_path = fs.path.join(data_dir.path, "notfounddir")
	local success, result = fs.dir.try_remove(not_found_path)
	assert(success == false and result == "NotFound", "not found should be not found")
end

tryremovenotfound()

local function thisproject()
	local src_files = fs.dir.project():expect_dir("src"):list(true, function(path: string)
		return if string.match(path, "%.rs$") then true else false
	end)
	local tt = require("@extra/tt")
	src_files = tt.arraymap(src_files, function(_, v)
		return fs.path.child(v) :: string
	end)
	assert(
		table.find(src_files, "main.rs") ~= nil,
		"shoouild be able to find main.rs in the project src"
	)
end

thisproject()

local function dirhome()
	fs.dir.home():list(false)
end

dirhome()

local function cwd()
	fs.dir.cwd():expect_dir("src")
end

cwd()
