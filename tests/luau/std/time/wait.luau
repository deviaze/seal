local time = require("@std/time")

local function timewaits()
    local t0 = os.clock()
    time.wait(1)
    assert(1 - (os.clock() - t0) < 0.001, "we should have 1 ms precision")
end

timewaits()