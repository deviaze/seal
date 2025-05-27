--!strict
local json = require("@std/json")
local str = require("@std/str")

local function unindent(s: string): string
    return str.unindent(s, "Spaces", 4)
end

local function unencodable()
    local bad = {
        c = 5,
        d = {
            f = function() return "hi" end,
        }
    }
    local s, result = pcall(function()
        return json.encode(bad)
    end)
    assert(s == false, "encoding table w/ a function should not succeed")
    assert(
        tostring(result):match("cannot serialize <function>"), 
        "didn't get expected cannot serialize <function> err message"
    )
end

unencodable()

type CatOwnerData = {
    reality: boolean,
    name: string,
    age: number,
    cats: {
        { name: string, age: number }
    },
}

local function objectdecode()
    local data = [[
    {
        "name": "deviaze",
        "age": 23,
        "reality": false,
        "cats": [
            {
                "name": "Taz",
                "age": 8
            },
            {
                "name": "Nanuk",
                "age": 0.95
            }
        ]
    }
    ]]
    data = unindent(data)
    local de = json.decode(data) :: CatOwnerData
    assert(de.cats[1].age == 8, "Taz age not 8, was arbitrary precision enabled??")
end

objectdecode()

local function hasnull()
    local maybecat = [[
    {
        "name": "Taz",
        "children": null, // trailing commas and comment allowed
    }
    ]]
    maybecat = unindent(maybecat)
    local hmm = json.decode(maybecat)
    -- print(tostring(hmm.children))
    assert(hmm.children == json.null(), "null should be null")
end

hasnull()

local function serializenull()
    local d = {
        name = "hi",
        payload = json.null(),
    }
    local raw = json.raw(d)
    assert(raw == [[{"name":"hi","payload":null}]], "null not serializing properly?")
end

serializenull()

local function sortorder()
    local data = {
        hi = 2,
        bye = 3,
    }
    local encoded = json.encode(data, { sorted = true })
    local chars = str.split(encoded)
    assert(chars[6] == "b", "chars[6] not b? (start of bye)?")
end

sortorder()

local function encodearrays()
    local basic = {1, 2, 3, 4}
    assert(json.raw(basic) == "[1,2,3,4]", "simple array encoding not work?")
    local object = {}
    assert(json.raw(object) == "{}", "empty table should be encoded as json object not array")
    local json_array = json.array()
    assert(json.raw(json_array) == "[]", "empty json.array encodes as array")
    table.insert(json_array, "hi")
    assert(json.raw(json_array) == `["hi"]`, "adding an element to json array should work")
    table.remove(json_array, 1)
    assert(json.raw(json_array) == "[]", "removing all elements from json array should continue being json array")
end

encodearrays()
