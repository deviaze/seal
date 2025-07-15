local time = require("@std/time")

print("o_o")

time.wait(0.75)

for _, saying in { "cats", "dogs", "seals" } do
    print(`got {saying}`)
    time.wait(0.45)
end
