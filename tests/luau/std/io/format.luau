local format = require("@std/io/format")

local t = setmetatable({
    x = 1,
    y = 2,
}, {
    __call = function(self: any, hmm: string): string
        return hmm .. "call"
    end
})

local pretty = format(t)
dp(pretty)
local debuggable = format.debug(t)
print(debuggable)