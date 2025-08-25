local datetime = require("@std/time/datetime")

local function nownow()
    local now = datetime.now()
end

nownow()

local function canparse()
    local dt = datetime.parse("2027-01-02 06:10", "ISO_8601", "US/Central")
    assert(dt.year == 2027, "should be 2027")
    assert(dt.month == 1, "should be january")
    assert(dt.day == 2, "should be the second")
    assert(dt.hour == 6, "should be the 6th hour")
    assert(dt.minute == 10, "should be the 10th minute")
    assert(dt.timezone == "US/Central", "IANA timezone should match what we put in")
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
    -- this also ensures .days is rounded to 4 decimal places
    assert(now:since(three_days_ago):duration().days == 3, "should be three days since 3 days ago")
end

cansub()

local function differenttimezones()
    local now = datetime.now()
    local cst = datetime.parse("2025-01-01 00:00", "ISO_8601", "US/Central")
    local utc =  datetime.parse("2025-01-01 00:00", "ISO_8601", "UTC")
    
    assert(utc:timespan(cst):duration().seconds == 21600, "timespan timezones not timezoning?")
    assert(utc:to(cst):duration().seconds == 21600, "to timezones not timezoning?")
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

local function timespans()
    local jan_01 = datetime.parse(`2025-01-01 00:00`, "ISO_8601", "UTC")
    local sept = datetime.parse(`2025-09-01 12:12`, "ISO_8601", "Africa/Cairo")
    local secs = sept:since(jan_01):duration().seconds
    assert(secs == 21028320, "seconds from sept 01 dont match")
end

timespans()

local function formatting()
    local jan_01 = datetime.parse(`2025-01-01 00:00`, "ISO_8601", "UTC")
    local formatted = jan_01:format("ISO_8601")
    assert(formatted == "2025-01-01 00:00", "formatting should iso 8601?")
end

formatting()

local function istypeof()
    assert(typeof(datetime.now()) == "DateTime", "typeof DateTime must be DateTime obviously")
end

istypeof()

local function changingtimezones()
    local rn = datetime.now()
    local rn_in_pst = rn:in_timezone("US/Pacific")
end

changingtimezones()

local function timestamping()
    local now = datetime.now()
    local in_ny = now:in_timezone("America/New_York")
    assert(now.unix_timestamp == in_ny.unix_timestamp, "unix timestamps shouldn't depend on tz")
end

timestamping()

local function addingrelativesgivesrelative()
    local rel1 = datetime.years(3, datetime.now()) + datetime.months(2) + datetime.days(1)
    assert(rel1.relative_to ~= nil, "relative + nonrelative + nonrelative -> relative")
end
addingrelativesgivesrelative()

local function addingyearsneedsrelativeto()
    local s, err = pcall(function()
        return datetime.years(1) + datetime.years(2)
    end)
    assert(typeof(err) == "error" and tostring(err):match("without a relative"), "should give without a relative error")

    local now = datetime.now()
    local two_years = datetime.years(1, now) + datetime.years(1)
    assert(two_years:duration().years == 2, "2 years should be 2 years fr")
end

addingyearsneedsrelativeto()

local function rn_mdy()
    local now = datetime.now()
    local now_mdy = now:format("MM/DD/YY")
end

rn_mdy()

local function braziltimeroundtrip()
    local dt1 = datetime.parse("2025-08-24T21:48:20-03:00", "RFC_3339", "AUTO")
    local dt2_timestamp = dt1:format("RFC_3339")
    local dt2 = datetime.parse(dt2_timestamp, "RFC_3339", "AUTO")
    assert(dt1 == dt2, "roundtrip brazil should be equal")
end

braziltimeroundtrip()