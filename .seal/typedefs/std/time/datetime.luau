local datetime = {
    common_formats = {
        ISO_8601 = "%Y-%m-%d %H:%M" :: "%Y-%m-%d %H:%M",
        RFC_2822 = "%a, %d %b %Y %H:%M:%S %z" :: "%a, %d %b %Y %H:%M:%S %z",
        RFC_3339 = "%Y-%m-%dT%H:%M:%S%:z" :: "%Y-%m-%dT%H:%M:%S%:z",
        SHORT_DATE = "%Y-%m-%d" :: "%Y-%m-%d",
        SHORT_TIME = "%H:%M" :: "%H:%M",
        FULL_DATE_TIME = "%A, %B %d, %Y %H:%M:%S" :: "%A, %B %d, %Y %H:%M:%S",
		["MM/DD/YYYY"] = "%m/%d/%Y" :: "%m/%d/%Y",
        ["MM/DD/YYYY HH:MM (AM/PM)"] = "%m/%d/%Y %I:%M %p" :: "%m/%d/%Y %I:%M %p",
        ["MM/DD/YY"] = "%m/%d/%y" :: "%m/%d/%y",
        ["HH:MM (AM/PM)"] = "%I:%M %p" :: "%I:%M %p",
        AMERICAN_FULL_DATE_TIME = "%A, %B %d, %Y %I:%M:%S %p" :: "%A, %B %d, %Y %I:%M:%S %p",
    }
}

export type DateTimeLocal = {
	unix_timestamp: number,
	--- format a DateTime with Rust chrono format string
	format: (self: DateTimeLocal, format: string) -> string,
}

export type DateTimeUtc = {
	unix_timestamp: number,
	--- format a DateTime with Rust chrono format string
	format_utc: (self: DateTimeUtc, format: string) -> string,
	format_local: (self: DateTimeUtc, format: string) -> string,
}


--[=[
Returns handle to a `chrono::Local` `DateTime`.

## Usage
```luau
local datetime = require("@std/time/datetime")
print(`the time is now {datetime.now():format(datetime.common_formats["HH:MM (AM/PM)"])}`)
print(`in unix, this means {datetime.now().unix_timestamp}`)
```
]=]
function datetime.now(): DateTimeLocal
	return nil :: any
end

function datetime.from(unix_timestamp: number): DateTimeUtc
    return nil :: any
end

return datetime
