local time = require("@std/time")

local days = (time.days(10) - time.days(40)):abs()

local small = time.microseconds(100)
print(small)