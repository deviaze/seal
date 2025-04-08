local fs = require("@std/fs")
local base64 = require("@std/serde/base64")

local nanuk_buffy = fs.readbytes("./tests/data/nanuk.png")

local sensible_nanuk = base64.encode(nanuk_buffy)

local meow_page = fs.readfile("./tests/data/server-views/index.html")
local nanuk_encoded_64 = base64.encode(fs.readbytes("./tests/data/nanuk.png"))