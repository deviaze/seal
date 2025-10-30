local input = require("@std/io/input")
local prompt = require("@std/io/prompt")

local animals: { string } = {}
print("Animals to pick from? (ctrlc when done to stop)")
local index = 1
while true do
    local response = input.readline(`animal {index}: `)
    if typeof(response) == "interrupt" then
        break
    elseif typeof(response) == "string" then
        table.insert(animals, response)
        index += 1
    end
end

local default = tonumber(prompt.text("default (index)?"))

local result = prompt.pick("Favorite animal?", animals, default)

print(`you picked {animals[result]}`)