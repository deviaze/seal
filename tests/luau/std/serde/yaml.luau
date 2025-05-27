local yaml = require("@std/serde/yaml")
local path = require("@std/fs/path")
local null = require("@std/json").null()

local content = {
	why = 1,
	am = 2,
	i_like = 3,
	this = {1, 2, 3, 4, "hi"},
	are = {
		you = {
			too = 1
		}
	}
}

local example_yaml_path = path.join(".", "tests", "data", "example_yaml.yaml")

local function basicserde()
	local encoded = yaml.encode(content)
	assert(string.match(encoded, "why: 1"), "why: 1 not found, we running into serde error?")
end

basicserde()

local function readfile()
	local example = yaml.readfile(example_yaml_path)
	assert(example.object.array[1].null_value == null, "null after deserializing from yaml should eq null")
end

readfile()

local function writefile()
	local data_path = path.parent(example_yaml_path) :: string
	local whyamilikethispath = path.join(data_path, "whyamilikethis.yaml")
	yaml.writefile(whyamilikethispath, content)
end

writefile()