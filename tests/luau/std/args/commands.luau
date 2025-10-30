local str = require("@std/str")
local args = require("@std/args")
local colors = require("@std/colors")

local parsed = args.parse("lgti", "Let's get that imported!", {
    description = "Add libraries directly from GitHub without a full package management solution.",
    examples = {
        "add 'https://github.com/luaulover/mouseauto.git'",
        "add 'luaulover/mouseauto' mouseauto --folder='./dependencies'",
        "remove mouseauto",
    },
    footer = `See {colors.style.underline("https://github.com/deviaze/lgti")} for support & documentation`
}):commands(
    args.command("add", "Add/import a repository from GitHub"):aliases("a"):args(
        args.positional(
            "repo", 
            "Permalink to the GitHub repository to import, can either be the full link or just owner/repo"
        ),
        args.named("--rename", "Rename the repository after importing?"),
        args.named("--src", "Path (relative to repo root) to grab src from")
            :default("./src"),
        args.named("--folder", "Folder in this repository you want to import the repo to")
            :default("./libraries")
    ),
    args.command("remove", "Remove a repository added by lgti"):aliases("r"):args(
        args.positional("name", "Name of the repo to remove")
    )
)

if parsed.command == "add" then
    print("Got command add")
    local repo = parsed:expect("repo") :: string
    print(`Got unparsed repo name {repo}`)
    local project_src = parsed:expect("src")

    local repo_name = "" do
        local repo_splits = str.split(repo, "/")
        repo_name = repo_splits[#repo_splits]
        repo_name = parsed:get("rename", repo_name) :: string
    end
    print(`Got parsed repo name {repo_name}`)

    local folder = parsed:expect("folder")
    print(`Got folder name {folder}`)
elseif parsed.command == "remove" then
    print("Got command remove")
    print(`Got repo name {parsed:expect("repo")}`)
end