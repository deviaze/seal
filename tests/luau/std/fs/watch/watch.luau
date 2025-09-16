local fs = require("@std/fs")
local env = require("@std/env")
local thread = require("@std/thread")
local format = require("@std/io/format")
local unformat = require("@std/io/output").unformat

local function watchiter()
    -- child thread calls fs lib fns to simulate 'reproducible' fs events
    local handle = thread.spawn {
        path = "./child.luau"
    }

    local start_time = os.clock()

    -- local event_categories: { fs.WatchEventCategory } = {}
    local event_categories: { string } = {} -- LUAU FIXME: table.insert bounds are broken

    local files_created: { string } = {}
    local creates_found = {
        files = 0,
        dirs = 0,
    }
    local removes_found = {
        files = 0,
        dirs = 0,
    }
    local writes_found = 0

    for cat, event in fs.watch(script:parent()) do -- watch only this ./tests/luau/std/fs/watch dir
        if cat == "Access" then
            continue -- these r just noise
        elseif cat == "None" then
            if os.clock() - start_time > 8 then
                break
            else
                continue
            end
        end

        print({
            event = cat,
            path = event.paths[1],
            kind = event.kind,
        })

        if cat == "Create" then
            if event.kind == "Create::Directory" then
                creates_found.dirs += 1
            elseif event.kind == "Create::File" then
                -- creates are unreliable!
                creates_found.files += 1
                for _, f in event.paths do
                    table.insert(files_created, f)
                end
            elseif env.os == "Windows" then
                creates_found.dirs += 1
                creates_found.files += 1
            end
        elseif event.is_write then
            writes_found += 1
            if not table.find(files_created, event.paths[1]) then
                creates_found.files += 1
            end
        elseif cat == "Modify::Other" then
            writes_found += 1
        elseif cat == "Remove" then
            if event.kind == "Remove::File" then
                removes_found.files += 1
            elseif event.kind == "Remove::Directory" then
                removes_found.dirs += 1
            elseif event.kind == "Remove::Any" and env.os == "Windows" then
                removes_found.files += 1
                removes_found.dirs += 1
            else
                -- note warn prints to stderr and therefore should fail ci

                warn(
                    `got remove that wasn't file or dir on platform {env.os}; got: {format(event)}`
                )
            end
        end

        table.insert(event_categories, cat)
    end

    -- make sure we didn't wait too long or block longer than expected
    -- we want to make sure if the loop actually should break after 8 seconds nothing blocks it
    -- from exiting around 8.0 -> 8.5ish seconds in
    local now = os.clock() - start_time
    assert(now > 8 and now < 9, `expected loop to exit during its 8-9th second, got: {now}`)

    handle:join()

    local formatted_events = unformat(format(event_categories))

    assert(creates_found.files >= 2, `at least 2 files should be created (testfile.txt and taz.json), got: {format(creates_found)} | events {formatted_events}`)
    -- looks like macos makes multiple create calls per dir ?? or something
    assert(creates_found.dirs >= 1, `only one directory (cats) should've been created, got: {format(creates_found)} | events {formatted_events}`)
    assert(removes_found.dirs >= 1, "1 directory should've been removed: cats")
    assert(removes_found.files >= 1, "at least 1 file should've been removed (content.txt; taz.json might be removed w/ parent dir)")
    -- macos puts most writes as creates i guess?
    assert(writes_found >= 3 or creates_found.files >= 3, "at least 3 files should've been modified (testfile.txt twice, taz.json once)")
end
watchiter()
