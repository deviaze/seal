local time = {}

--- Waits `seconds` in the current program thread (implemented with Rust's `thread::sleep`)
function time.wait(seconds: number): true
	return nil :: any
end

time.datetime = require("@self/datetime")

type time = typeof(time)

return time