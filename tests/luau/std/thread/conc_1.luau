-- this should be a subscript/thread

local time = require("@std/time")

local reps = 0
while true do
	time.wait(0.25)
	reps += 1
	print("heh")
	if reps > 24 then
		break
	elseif reps == 18 then
		error("how break traceback?")
	end
end

print("done")