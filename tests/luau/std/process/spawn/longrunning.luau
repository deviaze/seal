-- example of a long running process that doesnt instantly terminate
local time = require("@std/time")
local args = require("@std/env").args

print("Starting...")

while time.wait(0.125) do
	local x = math.random(1, tonumber(args[1]) or 125)
	print(x)
	if x > (tonumber(args[2]) or 115) then
		break
	end
end

print("Stopping")