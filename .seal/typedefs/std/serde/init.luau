--[=[
Lib for serializing/deserializing/encoding/decoding different things.
]=]
local serde = {}

serde.json = require("@std/json")

serde.base64 = require("@std/serde/base64")

serde.toml = require("@std/serde/toml")

serde.yaml = require("@std/serde/yaml")

return serde