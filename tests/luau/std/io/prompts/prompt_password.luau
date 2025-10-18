local prompt = require("@std/io/prompt")

local pass = prompt.password("ssh key?")
print(pass)

local passask = prompt.password("number of children", "*")
print(passask)