local args = require("@std/args")

local parsed = args.parse("gsw", "a simple fuzzy git branch switcher", {
    examples = {
        "main",
        "mybranch --create",
        "mybr"
    },
    footer = "written with seal in luau by deviaze :)"
}):simple(
    args.positional("branch", "any part of the branch you want to switch to, as long as it's a substring match")
        :optional(),
    args.flag("--confirm", "show a confirmation prompt before switching"),
    args.flag("--create", "allow creation of new branches"):aliases("-c")
)

local branch = parsed:get("branch") :: string?
if branch then
    print(`Switching to branch {branch}`)
else
    print(`Listing all branches:`)
end

if parsed.flags["--confirm"] then
    print("Should show confirmation prompt")
end
if parsed.flags["--create"] then
    print("Should allow branch creation")
end
