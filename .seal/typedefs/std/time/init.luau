-- local time = {}

-- --- Waits `seconds` in the current program thread (implemented with Rust's `thread::sleep`)
-- function time.wait(seconds: number): true
-- 	return nil :: any
-- end

-- time.datetime = require("@self/datetime")

-- type time = typeof(time)

-- return time

type time = {
    --- Blocks the current VM for approximately `seconds`, accurate to millisecond-ish precision.
    --- Implemented with Rust's `thread::sleep`.
    wait: (seconds: number) -> true,

    --- `DateTime` and `TimeSpan` libraries.
    datetime: typeof(require("@self/datetime")),

    --- Constructs a `Duration` representing `y` years. This assumes years are `365` days long.
    years: (y: number) -> Duration,

    --- Constructs a `Duration` representing `months` months. This assumes months are 365 / 12 days long.
    --- <br> For accurate months, use `TimeSpan`s instead.
    months: (months: number) -> Duration,

    --- Constructs a `Duration` representing `d` days.
    days: (d: number) -> Duration,

    --- Constructs a `Duration` representing `h` hours.
    hours: (h: number) -> Duration,

    --- Constructs a `Duration` representing `m` minutes.
    minutes: (m: number) -> Duration,

    --- Constructs a `Duration` representing `s` seconds.
    seconds: (s: number) -> Duration,

    --- Constructs a `Duration` representing `ms` milliseconds.
    milliseconds: (ms: number) -> Duration,

    --- Constructs a `Duration` representing `us` microseconds.
    microseconds: (us: number) -> Duration,

    --- Constructs a `Duration` representing `n` nanoseconds. This is accurate only up to ~285 years (expressed in nanoseconds).
    nanoseconds: (n: number) -> Duration,
}

return {} :: time