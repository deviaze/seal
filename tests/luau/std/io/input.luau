--!nolint DeprecatedApi
local input = require("@std/io/input")

local response = input.get("whats ur user: ")
print(`your user is: {response}`)