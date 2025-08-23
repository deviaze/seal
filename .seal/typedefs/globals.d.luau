declare function pcall<T>(protected_callback: (any) -> T): (boolean, T | error)

declare function p<T...>(...: T...): T...

declare function pp<T...>(...: T...): T...

declare function dp<T...>(...: T...): T...

declare function warn<T...>(...: T...): nil

declare _SEAL_VERSION: string

declare script: {
	entry_path: string,
	context: string?,
	path: (self: any) -> string,
	parent: (self: any) -> string,
}

declare channel: {
	send: <D>(self: any, data: D | string) -> (),
	sendbytes: <D>(self: any, data: buffer) -> (),
	read: <D>(self: any) -> D?,
	read_await: <D>(self: any) -> D,
	readbytes: (self: any) -> buffer?,
	readbytes_await: (self: any) -> buffer,
	data: any?,
}?

--- an error returned from mluau, can be tostringed
declare class error end

type COMMON_FORMATS =
	| "ISO_8601"
	| "RFC_2822"
	| "RFC_3339"
	| "SHORT_DATE"
	| "SHORT_TIME"
	| "FULL_DATE_TIME"
	| "LOGGING_24_HR"
	| "LOGGING_12_HR"
	| "MM/DD/YY"
	| "MM/DD/YYYY HH:MM (AM/PM)"
	| "MM/DD/YY"
	| "HH:MM (AM/PM)"
	| "AMERICAN_FULL_DATE_TIME"

declare class DateTime
	year: number
	month: number
	day: number
	hour: number
	minute: number
	second: number
	millisecond: number
	weekday: string
	unix_timestamp: number
	timezone: string
	iso: string

	function timespan(self, other: DateTime): TimeSpan
	function since(self, other: DateTime): TimeSpan

	function format(self, format: string | COMMON_FORMATS): string
	function display(self): string
	function rfc_2822(self): string
	function rfc_3339(self): string
	function rfc_3339_opts(self): string
	function iso_8601(self): string
	function http_date(self): string

	function __add(self, span: TimeSpan): DateTime
	function __sub(self, span: TimeSpan): DateTime
	function __eq(self, other: DateTime): boolean
    function __lt(self, other: DateTime): boolean
    function __le(self, other: DateTime): boolean
end

declare class TimeSpan
	years: number
	months: number
	days: number
	hours: number
	minutes: number
	seconds: number
	milliseconds: number
	microseconds: number

	function duration(self): Duration
	function __add(self, other: TimeSpan): TimeSpan
	function __sub(self, other: TimeSpan): TimeSpan
end

declare class Duration
    days: number
    hours: number
    minutes: number
    seconds: number
    milliseconds: number
    microseconds: number
    nanoseconds: number -- disabled due to overflow issue

	function abs(self): Duration
    function __add(self, other: Duration): Duration
    function __sub(self, other: Duration): Duration
    function __eq(self, other: Duration): boolean
    function __lt(self, other: Duration): boolean
    function __le(self, other: Duration): boolean
    function display(self): string
end
