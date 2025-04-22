local fs = require("@std/fs")
local env = require("@std/env")
local file = require("@std/fs/file")
local path = require("@std/fs/path")

local permission_denied_path = 
	if env.os == "Windows" then [[C:\System Volume Information]]
	elseif env.os == "Linux" then "/root"
	elseif env.os == "Android" then "/data"
	elseif env.os == "MacOS" then "/private/var/root"
	else "/root"

local datadir = path.join("./", "tests", "data")

local function tryreadok()
	local content, result = file.try_read(path.join(datadir, "cats.json"))
	assert(result == "Ok", "why reading cats.json not Ok?")
	assert(content ~= nil, "why content is nil?")
	assert(content:match('"reality": false'), "why is read content's reality not false?")
end

tryreadok()

local function tryreadnonexistent()
	local content, result = file.try_read("./mowtsra.txt")
	assert(content == nil, "why do we have a file named mowtsra.txt?")
	assert(result == "NotFound", "file should not be found")
end

tryreadnonexistent()

if env.os ~= "Windows" and env.os ~= "Android" then
	local function tryreadpermissiondenied()
		local test_path = path.join(permission_denied_path, "hi")
		local s, f = pcall(function()
			return path.exists(test_path)
		end)
		assert(tostring(f):match("permission denied"), "permission denied path should be permission denied?")
		local content, result = file.try_read(test_path)
		assert(content == nil, "why can we read permission denied path and why is there a file named hi there?")
		assert(result == "PermissionDenied", "why can we read permission denied path?")
	end
	
	tryreadpermissiondenied()
end

local function tryreadbytes()
	local cats, result = file.try_readbytes(path.join(datadir, "cats.json"))
	assert(cats ~= nil, "we shouldn't have had an issue trying to read cats.json")
	assert(result == "Ok", "readbytes result should be Ok")
end

tryreadbytes()

local function tryreadbytesnotfound()
	local content_buffy, result = file.try_readbytes("./hehe.json")
	assert(content_buffy == nil, "shouldnt be able to readbytes hehe.json")
	assert(result == "NotFound", "why do we have hehe.json??")
end

tryreadbytesnotfound()

if env.os ~= "Windows" and env.os ~= "Android" then
	local function tryreadbytespermissiondenied()
		local test_path = path.join(permission_denied_path, "hehe")
		local content_buffy, result = file.try_readbytes(test_path)
		assert(content_buffy == nil, "we shouldn't have content from trying to readbytes permission denied path")
		assert(result == "PermissionDenied", "readbytes permission denied path should PermissionDenied")
	end
	
	tryreadbytespermissiondenied()
end

local function trywrite()
	local new_file_path = path.join(datadir, "hmm.txt")
	local success, result = file.try_write(new_file_path, "trywritesayshmm")
	assert(success == true, "writing file there should've worked")
	assert(fs.readfile(new_file_path) == "trywritesayshmm", "try_write text doesnt match fs.readfile")
	fs.removefile(new_file_path)
end

trywrite()

local function trywritepermissiondenied()
	local test_path = path.join(permission_denied_path, "cats.json")
	local cats_content: string?, cats_result = fs.file.try_read(path.join(datadir, "cats.json"))
	if cats_content and cats_result == "Ok" then
		local success, result = file.try_write(test_path, cats_content)
		assert(success == false, "shouldn't be able to write to permission denied path")
		assert(result == "PermissionDenied", "incorrect result in trying to write to permission denied path")
	else
		error("we shouldn't have an issue try_reading cats.json")
	end
end

trywritepermissiondenied()

local function tryremove()
	local rmpath = path.join(datadir, "normalfile.txt")
	fs.writefile(rmpath, "hi")
	local success, status  = file.try_remove(rmpath)
	assert(success == true and status == "Ok")
end

tryremove()

local function tryremovefileadir()
	local data_dir = fs.dir.from(datadir)
	local cantremoveme = data_dir:add_tree(
		"tryfilecantremoveme", 
		fs.tree()
			:with_file("init.luau", "return 'hi'")
	):expect_dir("tryfilecantremoveme")

	local success, result = file.try_remove(cantremoveme.path)
	if env.os == "Linux" or env.os == "MacOS" then
		assert(success == false and result == "IsADirectory", "unexpectedly not IsADirectory")
	elseif env.os == "Windows" then
		assert(success == false and result == "PermissionDenied", "removing dir with file.try_remove not PermissionDenied on Windows?")
	end
	
	cantremoveme:remove()
end

tryremovefileadir()

local function tryremovenotexists()
	local data_dir = fs.dir.from(datadir)
	local success, result = file.try_remove("unexpected_meow")
	assert(success == false and result == "NotFound", "why is there a file unexpected_meow")
end

tryremovenotexists()

if env.os ~= "Windows" and env.os ~= "Android" then
	local function tryremovepermissiondenied()
		local denied_path = path.join(permission_denied_path, "hi.txt")
		local success, failure = file.try_remove(denied_path)
		assert(success == false and failure == "PermissionDenied", "can remove permission denied? huh")
	end
	
	tryremovepermissiondenied()
end
