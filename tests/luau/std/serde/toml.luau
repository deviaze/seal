local toml = require("@std/serde/toml")
local str = require("@std/str")

local function readfile()
	local stuff = toml.readfile("./Cargo.toml")
	assert(stuff.dependencies.toml ~= nil, "Cargo.toml doesn't have toml as dependency? (can't read toml?)")
end

readfile()

local function basicserde()
	local cats = {
		Taz = {
			some = "cat",
			somenumber = 12.25,
			array = {1, 2, 3, "meow"}
		}
	}

	local encoded = toml.encode(cats)
	local lines = str.splitlines(encoded)
	assert(str.startswith(lines[2], [[array = [1, 2, 3, "meow"]]), "array not serialized correctly?")

	local backtoluau = toml.decode(encoded)
	assert(
		backtoluau.Taz.somenumber == 12.25,
		"number not deserialized correctly?"
	)
end

basicserde()