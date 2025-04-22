local fs = require("@std/fs")
local rsa = require("@std/crypt/rsa")

local keys = rsa.generatekeys()

local data_dir = fs.dir.from("./tests/data")
	:add_file("public.key", keys.public)
	:add_file("private.key", keys.private)

local encrypted = rsa.encrypt("whoami", keys.public)
local decrypted = rsa.decrypt(encrypted, keys.private)

assert(decrypted == "whoami", "rsa plaintext doesnt match")

data_dir:expect_file("private.key"):remove() -- so git guardian doesn't angery
data_dir:expect_file("public.key"):remove() -- so we don't git status changed every time we run tests