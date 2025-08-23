local time = require("@std/time")
local datetime = require("@std/time/datetime")

local function nownow()
    local now = datetime.now()
end

nownow()

local function canparse()
    local dt = datetime.parse("2027-01-02 06:10", "ISO_8601", "US/Central")
    -- print(dt.weekday)
end

canparse()

local function canadd()
    local now = datetime.now()
    local three_days = datetime.days(3)
    local three_days_from_now = now + three_days

    local one_monthish = datetime.days(30)
    local next_month = now + one_monthish
end
canadd()

local function cansub()
    local now = datetime.now()
    local three_days_ago = datetime.now() - datetime.days(3)
end

cansub()

local function differenttimezones()
    local now = datetime.now()
    local cst = datetime.parse("2025-01-01 00:00", "ISO_8601", "US/Central")
    local utc =  datetime.parse("2025-01-01 00:00", "ISO_8601", "UTC")
    
    assert(utc:timespan(cst):duration().seconds == 21600, "timezones not timezoning?")
end

differenttimezones()

local function simplecomparisons()
    local now = datetime.now()
    local five_days_ago = now - datetime.days(5)
    assert((now < five_days_ago) == false, "now cannot be < five days ago the world is broken")
    assert(now > five_days_ago, "now should be greater than five days ago")

    local feb2 = datetime.parse("02/02/2025 10:00", "%m/%d/%Y %H:%M", "UTC")
    local feb2_02 = datetime.parse("02/02/2025 10:00", "%m/%d/%Y %H:%M", "UTC")
    feb2_02 -= datetime.seconds(1)
    assert(feb2_02 ~= feb2, "minusing 1 second should cause false ==")
    feb2_02 += datetime.seconds(1)
    assert(feb2 == feb2_02, "minusing and plussing 1 second to feb2_02 should be == feb2")
end

simplecomparisons()