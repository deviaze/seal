local fs = require("@std/fs")
local time = require("@std/time")
local tp = fs.path.join(script:parent(), "testfile.txt")
if channel then
    fs.file.try_remove(tp) -- Remove::File
    time.wait(1)
    fs.writefile(tp, "content") -- Modify::Data & Close
    time.wait(1)
    local c = fs.readfile(tp) -- apparently nothing on linux?
    time.wait(0.125)
    fs.writefile(tp, "cats") -- Modify::Data & Close
    time.wait(1)
    fs.file.try_remove(tp) -- Remove::File
    time.wait(0.25)
    local cats_dir = fs.dir.ensure( -- Create::Directory
        fs.path.join(fs.path.parent(tp) :: string, "cats")
    )
    -- is this somehow a Modify::Data?? instead of a Create::File and then Modify::Data?
    cats_dir:add_file("taz.json", [[{"name":"Taz"}]]) -- & Close
    time.wait(0.16)
    cats_dir:remove() -- 1 or 2 Removes
end
