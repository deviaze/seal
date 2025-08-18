local fs = require("@std/fs")
local str = require("@std/str")
local env = require("@std/env")
local input = require("@std/io/input")
local process = require("@std/process")

local function prompt(message: string): string
    return str.trim(input.get(message)):lower()
end

-- move seal to ~/.local/bin/seal(.exe)
local move_here = fs.path.join(fs.path.home(), ".local", "bin")
local local_bin_dir = fs.dir.ensure(move_here, true) -- creates missing directories
local new_seal_location = local_bin_dir:join(if env.os == "Windows" then "seal.exe" else "seal")
-- ask user before moving
if prompt(`Move seal from {env.executable_path} to {new_seal_location}? (y/n): `) == "y" then
    fs.copy(env.executable_path, new_seal_location)
end

if env.os == "Windows" then
    print(`You need to add this to your powershell $PROFILE: $env:Path += ";{local_bin_dir.path}"`)
    local result = prompt("open $PROFILE in vscode? (Y/n): ")
    if result == "y" or result == "" then
        -- open vscode at your $PROFILE so you can easily modify 
        -- $env:Path += ";<PATH TO DIR SEAL IS IN>"
        process.shell("Start-Process code -ArgumentList $PROFILE")
    end
else
    print(`If you moved ./seal to ~/.local/bin/seal, seal should be available in PATH (or you can trivially add it to your .zshrc/.bashrc/etc.)`)
end
