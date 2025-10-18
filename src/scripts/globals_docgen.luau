-- writing docs for our globals in ./seal/typedefs/docs.json is absolute pain
-- so we do it with seal instead lol
--[[
    how docfiles work (according to my investigation)
    see https://raw.githubusercontent.com/JohnnyMorganz/luau-lsp/refs/heads/main/scripts/api-docs/en-us.json

    1. @roblox is hardcoded (i dont like it either, see: https://github.com/JohnnyMorganz/luau-lsp/issues/971)
    2. globaltype refers to a declare class/extern type
    3. global refers to a declare global
    4. if you want docs on on keys/members/params you have to
    put them as separate top level entries @roblox/globaltype/DateTime.timespan

]]

local fs = require("@std/fs")
local json = require("@std/json")
local str = require("@std/str")

type ExternTypeDoc = {
    documentation: string,
    keys: {
        [string]: ExternTypeDoc,
    }?,
    params: {
        [string]: ExternTypeDoc,
    }?,
}

local function un3(s: string)
    return str.unindent(s, "Spaces", 4 * 3)
end
local function un5(s: string)
    return str.unindent(s, "Spaces", 4 * 5)
end

type ExternTypeDocs = {
    [string]: ExternTypeDoc
}

local extern_types: ExternTypeDocs = {
    ["error"] = {
        documentation = un3 [[
            A wrapped error returned by mluau's `pcall` or seal, can be tostringed.

            To interact with errors (and make your own), use `@std/err`

            ## Usage

            ```luau
            local success, result = pcall(myfn)
            if typeof(result) == "error" then
                print(tostring(result)) -- result is an error message
            else
                -- result should be narrowed to the return type of myfn
            end
            ```
        ]]
    },
    interrupt = {
        documentation = un3 [[
            An intercepted user interrupt, either Ctrl-C or Ctrl-D.
        ]],
        keys = {
            code = {
                documentation = "What caused this interrupt (either CtrlC or CtrlD)",
            }
        },
    },
    DateTime = {
        documentation = un3 [[
            Represents a Timezone-aware moment in date and time.
            
            - To make a new `DateTime`, use `@std/time/datetime`'s `now` or `parse` functions.
            - Use `DateTime:format` to format a datetime by passing in a strftime format string or COMMON_FORMAT.
            - To do arithmetic on a `DateTime`, add or subtract a `TimeSpan` from it.
            - Use `DateTime:since`, `DateTime:to`, and `DateTime:timespan` to get `TimeSpan`s between two `DateTime`s.
            - You can also compare `DateTime`s using the standard `>`, `<`, `<=`, `>=`, and `==` operators.

            ## Usage
            
            ```luau
            local datetime = require("@std/time/datetime")
            local yesterday = datetime.now() - datetime.days(1)
            print(yesterday:format("ISO_8601"))
            ```
            ]],
        keys = {
            unix_timestamp = {
                documentation = "Time, in seconds, since the Unix epoch (January 1st, 1970 at 00:00 UTC)"
            },
            timezone = {
                documentation = "The IANA timezone for this DateTime. All DateTimes *must* have a timezone!"
            },
            format = {
                documentation = un5 [[
                    Format this `DateTime` with strftime format strings or COMMON_FORMATs.

                    ## Usage
                    ```luau
                    local datetime = require("@std/time/datetime")
                    local now = datetime.now()
                    print(now:format("ISO_8601")) --> 2025-08-23 14:57
                    print(now:format("%A")) --> Saturday
                    ```

                    ## Valid strftime format specifiers (Rust Jiff-compatible):
                    - %a  - Abbreviated weekday name (e.g., Sun)
                    - %A  - Full weekday name (e.g., Saturday)
                    - %b  - Abbreviated month name (e.g., Aug)
                    - %B  - Full month name (e.g., August)
                    - %c  - Locale-specific date and time (e.g., Sat Aug 23 15:01:00 2025)
                    - %d  - Day of the month (01-31)
                    - %H  - Hour in 24h format (00-23)
                    - %I  - Hour in 12h format (01-12)
                    - %j  - Day of the year (001-366)
                    - %m  - Month as a number (01-12)
                    - %M  - Minute (00-59)
                    - %p  - AM or PM
                    - %Q  - IANA time zone name (e.g., America/Chicago)
                    - %S  - Second (00-59)
                    - %U  - Week number of the year (Sunday as first day of week)
                    - %w  - Weekday as a number (0-6, Sunday is 0)
                    - %W  - Week number of the year (Monday as first day of week)
                    - %x  - Locale-specific date (e.g., 08/23/25)
                    - %X  - Locale-specific time (e.g., 15:01:00)
                    - %y  - Year without century (00-99)
                    - %Y  - Year with century (e.g., 2025)
                    - %%  - Literal percent sign
                ]]
            },
            timespan = {
                documentation = un5 [[
                    Get the `TimeSpan` between this DateTime and another. 
                    
                    If `other` is a DateTime in the future, the result is a positive `TimeSpan`, otherwise it'll be negative (recommend using `DateTime:since` instead). 
                    
                    Note that because `TimeSpans` store seconds/minutes/days/etc. separately, you have to call `TimeSpan:duration()` to get a `Duration` first to figure out how many days/seconds/etc. are in between two DateTimes."
                ]],
            },
            to = {
                documentation = un5 [[
                    Get the `TimeSpan` between this DateTime and another (preferably in the future).

                    ## Usage
                    ```luau
                    local datetime = require("@std/time/datetime")
                    local now = datetime.now()
                    local jan_01 = datetime.parse(`{now.year + 1}-01-01 00:00`, "ISO_8601", "UTC")
                    print(`Days until next year: {math.round(now:to(jan_01):duration().days}`))
                    ```
                ]]
            },
            since = {
                documentation = un5 [[
                    Get the `TimeSpan` between this DateTime and another (preferably in the past).

                    ## Usage
                    ```luau
                    local datetime = require("@std/time/datetime")
                    local now = datetime.now()
                    local jan_01 = datetime.parse(`{now.year}-01-01 00:00`, "ISO_8601", "UTC")
                    print(`Days in year to date: {math.round(now:since(jan_01):duration().days}`))
                    ```
                ]]
            },
            in_timezone = {
                documentation = "Returns a new `DateTime` representing the same instant in time but in a different timezone."
            }
        }
    },
    TimeSpan = {
        documentation = un3 [[
            A calendar-aware Duration type, composed of individual separate time units.

            - To create a `TimeSpan`, use `@std/time/datetime`'s `hours/days/seconds/etc.` functions.
            - You use these to add/subtract with `DateTimes` to shift them forwards or backwards in time.
            - Use `TimeSpan:duration()` to get the total number of days/minutes/hours/etc. in the `TimeSpan`.

            ## Usage

            ```luau
            local datetime = require("@std/time/datetime")

            local last_week = datetime.now() - datetime.days(7)
            ```

            ## Warnings

            ### 1. You can't compare `TimeSpan`s.

            - `time.hours(24)` and `time.days(1)` are have equivalent `Duration`s but aren't the same `TimeSpan`.
            - Check the *jiff* documentation to see why they don't allow `Span` comparisons.

            ### 2. Specify `relative_to` for `datetime.months` or `datetime.years` for `TimeSpan` arithmetic.
            
            - These 2 are calendar aware and can change duration depending on the DateTime they're relative to.
            - If you're simply doing `datetime.now() + datetime.years(1)` you're fine.
            - But if you're adding `TimeSpans` to each other you need to do something like `datetime.months(2, datetime.now()) + datetime.days(3)`
        ]],
        keys = {
            relative_to = {
                documentation = "The `DateTime` this `TimeSpan` is relative to, or nil if it's not relative to any."
            },
            duration = {
                documentation = "Convert this `TimeSpan` into a `Duration` so you can easily get total number of years/months/etc. in the `TimeSpan`."
            },
            units = {
                documentation = "Get the individual units in the internal representation of this `TimeSpan`. This is probably not what you want."
            }
        }
    },
    Duration = {
        documentation = un3 [[
            Represents a signed duration of time. Make these by calling functions from the `@std/time` library.
        ]]
    }
}

local globals: ExternTypeDocs = {
    ["_SEAL_VERSION"] = {
        documentation = "seal's version"
    },
    ["_VERSION"] = {
        documentation = "version of both Luau (best that we know, since luau doesn't officially publish _VERSION at runtime) and seal",
    },
    ["pcall"] = {
        documentation = un3 [[
            Call `protected_callback` safely without blowing up if it errors.
            <br>
            Unlike base Luau, seal returns an `error` type when `pcall` fails, allowing for slightly better error handling.

            ## Usage

            ```luau
            local success, result = pcall(function(): string -- return type when it succeeds
                return maybestringmaybekaboom()
            end)

            if typeof(result) == "error" then
                print(`got error: {result}`)
            else
                -- result should be narrowed to `string` here
            end
            ```

            ### I don't want to return anything

            - Set the return type to `nil` instead of `()`

            ### Multirets

            - Should be supported at runtime, but you probably want to cast `pcall` to `any` to appease the typechecker.
        ]]
    },
    ["pp"] = {
        documentation = "Pretty print for pretty tables (also returns, unlike `print`)."
    },
    ["p"] = {
        documentation = "Simple print (no colors, less formatting) and returns."
    },
    ["dp"] = {
        documentation = un3 [[
            Debug print; use this to investigate pesky non-utf8 codes in strings, see the source/line/function of
            things causing trouble, etc. Also returns what it prints.
            <br>
            Add a `__dp` function to your custom type's metatable to customize its debug print output.
        ]]
    },
    ["warn"] = {
        documentation = "Prints a yellow warning to stderr.",
    },
    ["script"] = {
        documentation = un3 [[
            The file you're currently working on. Has the extremely useful method `:parent()`
            which you can use to get the directory your file resides in.
        ]],
        keys = {
            entry_path = {
                documentation = "Path to the file or executable the user ran with seal ./myfile.luau, seal run, etc."
            },
            path = {
                documentation = "Path to the current script, also internally used as the chunk/debug name for this file in seal."
            },
            parent = {
                documentation = un5 [[
                    Path to this file's parent in the filesystem. Very useful if you want to write a script that saves something right next to it (tests!!)

                    ## Usage
                    
                    ```luau
                    local fs = require("@std/fs")
                    fs.writefile(fs.path.join(script:parent(), "meow.txt"), "meow")
                    ```
                ]]
            }
        }
    },
    ["channel"] = {
        documentation = un3 [[
            A global that exists in child threads.
            <br>
            Access startup data passed by the parent thread with `channel.data`

            ## Message passing
            - Send a data message back to parent thread: `channel:send` with automatic (json) serialization
            - Send a buffer back to the parent thread: `channel:sendbytes`
            - Read messages from the parent thread: `channel:read` and `channel:read_await`
            - Read a buffer message sent from the parent thread: `channel:readbytes` and `channel:readbytes_await`

            See `@std/thread`'s `JoinHandle` docs for more information. Channel methods are basically the same as their `JoinHandle` equivalents.

            ## Usage

            ```luau
            if channel then -- make sure we're in a child thread
                local data = channel.data
                local result: { string } = {}
                if data then
                    -- logic
                end
                channel:send(result)
            end
            ```
        ]]
    },
}

local luau_doc_overrides: ExternTypeDocs = {
    ["table"] = {
        documentation = un3 [[
            Luau's builtin table library, mostly focused on array-like table manipulation. 
            
            - Use `table.insert` to append or insert elements, 
            - `table.find` to check if/where an element exists in an array,
            - `table.remove` to remove elements (watch the footgun w/ `table.find`),
            - `table.concat` to concatenate `{ string }`s into strings. This is the fastest way to concat strings.
        ]],
        keys = {
            concat = {
                documentation = "Fastest way to concatenate strings in Luau. Don't do loops of ..=, just use this."
            },
            remove = {
                documentation = un5 [[
                    - `table.remove(t, index)` removes and returns the element at `index`
                    - `table.remove(t)` removes and returns the last element of your array (everyone considers this a mistake)

                    ## footgun

                    - Don't do the `table.remove(t, table.find(t, element))` footgun. 
                    - If the `table.find` call doesn't find anything, it returns `nil`, causing `table.remove` to pop the back element of your array `t` without warning.
                    
                    The Luau team can't fix this because of backwards bug-for-bug compatibility :/
                ]]
            }
        }
    }
}

type TransformedDocs = {
    [string]: {
        documentation: string,
        keys: {
            [string]: string,
        }?,
    }
}

local transformed: TransformedDocs = {}
local function transform(ext: ExternTypeDoc, name: string, globalwhat: "globaltype" | "global", prefix: "roblox" | "luau")
    local base = `@{prefix}/{globalwhat}/` .. name
    local key_names_paths: { [string]: string } = {}
    if ext.keys then
        for key, value in ext.keys do
            local key_name = base .. `.{key}`
            local key_docs = value.documentation
            if value.keys then
                error("ugh not yet impl")
            end
            transformed[key_name] = { documentation = key_docs }
            key_names_paths[key] = key_name
        end
    end
    transformed[base] = {
        documentation = ext.documentation,
        keys = key_names_paths
    }
end
for key, doc in extern_types do
    transform(doc, key, "globaltype", "roblox")
end
for key, doc in globals do
    transform(doc, key, "global", "roblox")
end
for key, doc in luau_doc_overrides do
    transform(doc, key, "global", "luau")
end

json.writefile(
    fs.path.join(".", ".seal", "typedefs", "docs.json"),
    transformed
)
