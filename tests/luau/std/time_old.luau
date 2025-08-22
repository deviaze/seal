-- local time = require("@std/time")
-- local datetime = require("@std/time/datetime")
-- local thread = require("@std/thread") -- thread.sleep does time too

-- local now = datetime.now()
-- local common_formats = datetime.common_formats

-- print(`the time is now {datetime.now():format(datetime.common_formats["HH:MM (AM/PM)"])}`)

-- local function nowfmt(format_name: string, format: string)
-- 	return `{format_name}: {now:format(format)}`
-- end

-- for format_name, format_string in common_formats :: { [string]: string } do 
-- 	print(nowfmt(format_name, format_string))
-- end

-- print("\n")

-- local t0 = os.clock()
-- thread.sleep(20)
-- print(`{os.clock() - t0} seconds for ~20 ms`)

-- local start_t = os.clock()
-- time.wait(2)
-- local elapsed = os.clock() - start_t
-- assert(math.round(elapsed) == 2, "wth why time not timing?")
-- print(`{elapsed} seconds for 2 seconds`)
