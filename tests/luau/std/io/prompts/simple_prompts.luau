local prompt = require("@std/io/prompt")
local colors = require("@std/colors")

local function text()
    local resp = prompt.text("what's your name")
    print(resp)
end

text()

local function colored()
    local resp = prompt.text(colors.bold.white("What's your name: "))
    print(resp)
end

colored()

local function confirm()
    if prompt.confirm("roses are red") then
        print("violets are blue")
    end
end

confirm()