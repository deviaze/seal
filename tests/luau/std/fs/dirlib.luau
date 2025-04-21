local fs = require("@std/fs")

local env = require("@std/env")
local permission_denied_path = 
	if env.os == "Windows" then [[C:\System Volume Information]]
	elseif env.os == "Linux" then "/root"
	elseif env.os == "Android" then "/data"
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

local function tryremovepermissiondenied()
	local denied_path = fs.path.join(permission_denied_path, "hi")
	local success, result, kind = fs.dir.try_remove(denied_path)
	assert(success == false and result == "PermissionDenied", "removing permisison denied dir should be permission denied")
end

tryremovepermissiondenied()

local function tryremovenotfound()
	local not_found_path = fs.path.join(data_dir.path, "notfounddir")
	local success, result = fs.dir.try_remove(not_found_path)
	assert(success == false and result == "NotFound", "not found should be not found")
end

tryremovenotfound()