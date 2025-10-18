local prompt = require("@std/io/prompt")
local try = require("@tests/try")

local result = try(function() 
    return prompt.pick("hi", { hi = 2 }) 
end):throws("options table is empty or")

