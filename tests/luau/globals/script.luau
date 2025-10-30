local fs = require("@std/fs")

local function parent()
    assert(
        fs.path.child(script:parent()) == "globals",
        "parent of this script should be globals"
    )
end
parent()

local function projectdir()
    local pr = script:project()
    assert(
        fs.path.child(pr) == "seal",
        "lol did you rename the repository (project dir expected to be seal)"
    )
end
projectdir()