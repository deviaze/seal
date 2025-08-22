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

    --- TimeSpan constructors and utilities
    datetime: typeof(require("@self/datetime")),

    --- Constructs a Duration representing `d` days.
    days: (d: number) -> Duration,

    --- Constructs a Duration representing `h` hours.
    hours: (h: number) -> Duration,

    --- Constructs a Duration representing `m` minutes.
    minutes: (m: number) -> Duration,

    --- Constructs a Duration representing `s` seconds.
    seconds: (s: number) -> Duration,

    --- Constructs a Duration representing `ms` milliseconds.
    milliseconds: (ms: number) -> Duration,

    --- Constructs a Duration representing `us` microseconds.
    microseconds: (us: number) -> Duration,

    -- nanoseconds: (ns: number) -> Duration -- disabled due to overflow issue
}

return {} :: time