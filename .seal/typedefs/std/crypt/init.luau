local crypt = {}

crypt.aes = require("@std/crypt/aes")
crypt.rsa = require("@std/crypt/rsa")
crypt.hash = require("@std/crypt/hash")
crypt.password = require("@std/crypt/password")

return crypt