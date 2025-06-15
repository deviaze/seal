-- example of a long running process that doesnt instantly terminate
local time = require("@std/time")

print("Starting...")

while time.wait(0.125) do
	local x = math.random(1, 121)
	print(x)
	if x > 65 then
		break
	end
end

print("Stopping")