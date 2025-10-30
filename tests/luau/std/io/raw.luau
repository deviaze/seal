local input = require("@std/io/input")
local time = require("@std/time")

local contents = ""

if input.tty() then
    input.rawmode(true)
    input.capture.paste(true)
    
    for event in input.events(time.milliseconds(100)) do
        if event.is == "Empty" then
            continue
        end
    
        if event.is == "Key" then
            if event.modifiers.ctrl and event.key == "c" then
                break
            end
        end
    
        if event.is == "Paste" then
            -- print(event.contents)
            contents = event.contents
            break
        end
    end
    
    input.capture.paste(false)
    input.rawmode(false)    
    print(`got contents:\n{contents}`)
end


