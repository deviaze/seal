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
    print(three_days_from_now:display())

    local one_monthish = datetime.days(30)
    local next_month = now + one_monthish
end
canadd()

local function differenttimezones()
    local now = datetime.now()
    local cst = datetime.parse("2025-01-01 00:00", "ISO_8601", "US/Central")
    local utc =  datetime.parse("2025-01-01 00:00", "ISO_8601", "UTC")
    
    assert(utc:timespan(cst):duration().seconds == 21600, "timezones not timezoning?")
end

differenttimezones()