--!strict
local json = require("@std/json")

local x = {
	c = 5,
	b = "meow",
}

local unencodable = {
	name = "deviaze",
	says = function()
		return "hi"
	end
}

local s, e = pcall(function()
	json.encode(unencodable)
end)

assert(s == false and tostring(e):match("cannot serialize <function>"))

local got_data = [[
{
  "name": "deviaze", 
  "age": 23, 
  "reality": false, 
  "cats": [{
    "name": "Taz",
	"age": 8
  }, {
    "name": "Nanuk",
	"age": 0.95
  }]
}
]]

type CatOwnerData = {
	reality: boolean,
	name: string,
	age: number,
	cats: {
		{ name: string, age: number }
	},
}

-- local t0 = os.clock()
local json_result = json.decode(got_data) :: CatOwnerData
-- print(os.clock() - t0)
-- print(json_result)

json.writefile("./tests/data/cats.json", json_result :: any)

-- print(fs.entries("./tests/data/"))
local deviazecats = json.readfile("./tests/data/cats.json")
-- print(deviazecats)


local t = {
	something = 2,
	nothing = 3,
	other = {
		something = 1,
		["21"] = { 1, 2, 3 }
	}
}

local encode_attempt_1 = json.encode(t, { pretty = true, sorted = true })
local encode_attempt_2 = json.encode(t, { pretty = true, sorted = true })

assert(encode_attempt_1 == encode_attempt_2, "not actually sorted in same order???")
