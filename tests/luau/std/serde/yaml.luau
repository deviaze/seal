local yaml = require("@std/serde/yaml")

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

yaml.writefile {
	path = "./tests/data/whyamilikethis.yaml",
	content = content
}

local back = yaml.readfile("./tests/data/whyamilikethis.yaml")

-- thanks for iteration iterating in same order on same device
assert(yaml.encode(back) == yaml.encode(content), "yamls dont match :/")

local y = yaml.readfile("./tests/data/example_yaml.yaml")
local re_y = yaml.encode(yaml.decode(yaml.encode(y)))
