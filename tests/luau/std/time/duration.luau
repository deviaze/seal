local time = require("@std/time")

local function daymath()
    local days = time.days(10) - time.days(40)
    assert(days.days == -30, "we should be -30 days now")
end

daymath()

local function minusing()
    local d = time.days(10) - time.seconds(200)
    assert(d.seconds == 863800, "should be 863800 seconds here")
end

minusing()

local function durationmadeofduration()
    local five_seconds = time.seconds(5)
    assert(typeof(five_seconds) == "Duration", "should be Duration")
end

durationmadeofduration()