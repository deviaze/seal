local prompt = require("@std/io/prompt")
local try = require("@tests/try")

-- local result = prompt.validate("are we human", nil :: any)
-- print(tostring(result))

local result = try(function()
    return prompt.validate("are we human", nil :: any)
end):throws("got: nil")

local n = prompt.validate("is a number", function(resp): string | true
    if tonumber(resp) then
        return true
    else
        return `Not a valid number: {resp}`
    end
end)
print(n)