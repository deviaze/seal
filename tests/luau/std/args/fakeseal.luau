local args = require("@std/args")
local err = require("@std/err")

local parsed = args.parse("seal", "the cutest luau runtime", {
    description = "A highly reliable scripting and automation-focused Luau runtime",
    examples = {
        "./myfile.luau" ,
        "run",
        "setup",
    },
    footer = "See the repository at deviaze/seal",        
}):commands(
    args.default(
        args.positional("file", "the luau file to run"),
        args.list("args", "arguments to pass to the file")
    ),
    args.command("run", "run the project at the cwd"):args(
        args.list("args", "arguments to pass to project")
    ),
    args.command("setup", "setup typedefs for a new project in the directory at the cwd"):args(
        args.positional("codebase", "codebase style: project or script")
            :validate(function(s): err.Result<string>
                if s == "project" or s == "p" then
                    return "project"
                elseif s == "script" or s == "s" then
                    return "script"
                else
                    return err.message("invalid codebase style")
                end
            end)
    )
)

if parsed.command == "default" then
    local filename = parsed:expect("file") :: string
    local args_to_pass = parsed:get("args") :: { string }
    print(`default filename {filename} with args {table.concat(args_to_pass, ", ")}`)
elseif parsed.command == "run" then
    print(`run with args {table.concat(parsed:get("args") :: { string }, ", ")}`)
elseif parsed.command == "setup" then
    local codebase_style = parsed:get("codebase")
    print(`codebase style {codebase_style}`)
end
