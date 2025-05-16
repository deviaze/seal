local meow = {
    fed = false,
    data = {
        ["/opt/idk/2"] = "safe",
    },
    cats = {
        Taz = 1,
        Nan = 2,
    },
    doer = function()
        
    end,
    [{ "i am a key" }] = true,
    [1.25] = 1,
    [vector.create(1, 2, 2312311.23)] = 12,
}
meow.a = meow
meow.cats.newcat = meow.cats
meow.meow = meow.cats.newcat
local buffy = buffer.create(32)
meow.b = buffy
local v = vector.create(0, 12, 2)
meow.v = v

local s, r = pcall(function()
    local fs = require("@std/fs")
    fs.readfile("we are an error. short days ago we were also error")
    return nil
end)

meow.err = r

local user = newproxy()

meow.u = user

local coro = coroutine.create(function()

end)

meow.coro = coro

local seen_tables = {}
-- print(process_raw_values(meow, seen_tables))
-- print(`print(meow.err): {tostring(meow.err)}`)

local prettyvalues = {
    a = 1,
    [1] = "hi",
    [2] = "meow",
    [3] = "stasrtawf",
    b = "3",
    [1.25] = 4,
    ["hello world"] = true,
    ["emowtsartatf"] = {},
    ["/opt/cats/1"] = true,
    [buffer.create(12)] = "hi",
    location = vector.create(1, 2, 4),
}

meow.pretty = prettyvalues

print(meow)

local a = {}
a.b = { [a] = a }
print(a)

local cats = {}
cats.Taz = {
  name = "Taz",
  age = 12,
}

cats.Nan = {
  name = "Nanuk",
  age = 1,
}

cats.catlist = {
  cats.Taz, cats.Nan
}

cats.self = cats

print(cats)

local children = {
	["folders 2"] = {
		{
			parts = {}
		}
	}
}

local nested_tables = {
	children = children,
	parts = children["folders 2"][1].parts,
	folders = children["folders 2"],
}

print(nested_tables)

print("hi")

print({ a=  "we", "are", "seal"})

print(vector.create(1, 2, 3))

local array_with_many_values = {}; for i = 1, 1001 do table.insert(array_with_many_values, i) end
-- print(array_with_many_values)

local fs = require("@std/fs")

local src = fs.readfile("./src/err.rs")

local with_newline_strings = {
	"tafwtfatfwtfwaw\n2321tsartfwat hrsdasrad \n \n 22312",
	cats = "nanuk\nTaz",
	src = src,
}
print(with_newline_strings)
