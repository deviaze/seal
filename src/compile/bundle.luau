--!optimize 2

-- yes, this is in luau.
local fs = require("@std/fs")
local str = require("@std/str")
local err = require("@std/err")
local luau = require("@std/luau")

local function get_entry_path(path: string): err.Result<string>
    if fs.file(path) then
        return path -- seal compile ./direct/path/to/file.luau -o myexec
    end

    local project_dir = fs.dir(path)
    if not project_dir then
        return err.message(`path at '{path} doesn't exist (so it can't be bundled)`)
    end

    local seal_dir = project_dir:find(".seal"):try_dir()
    if not seal_dir then
        return err.message(`directory at '{path}' isn't a seal project (it doesn't contain a .seal dir), so it cannot be bundled`)
    end

    local config_luau = seal_dir:find("config.luau"):try_file()
    if not config_luau then
        return err.message(`seal project at '{path}' missing its .seal/config.luau, it cannot be bundled because we don't know its entry path`)
    end

    local config = luau.eval(config_luau:read())
    if typeof(config) == "error" then
        return err.message(`config.luau at '{config_luau.path}' couldn't be evaluated due to err: {tostring(config)}`)
    elseif typeof(config) ~= "table" then
        return err.message(`config.luau at '{config_luau.path}' returned unexpected datatype; expected table, got: {typeof(config)}`)
    else
        config = config :: {
            entry_path: string?
        }
    end

    local found_entry_path = config.entry_path
    if not found_entry_path then
        return err.message(`config.luau at '{config_luau.path}' missing 'entry_path' field; we need a valid entry_path to bundle the project`)
    end

    found_entry_path = fs.path.normalize(project_dir:join(found_entry_path))

    local find_entry = fs.find(found_entry_path)
    if find_entry.type == "Directory" then
        local find_init_luau = find_entry:unwrap_dir():find("init.luau")
        find_entry = find_init_luau    
    end

    if find_entry.type == "File" then
        return find_entry.path
    elseif find_entry.type == "NotFound" then
        return err.message(`entry path at '{find_entry.path}' doesn't exist on the filesystem`)
    else
        return err.message(`entry_path at '{find_entry.path}' unexpectedly is a {find_entry.type}`)
    end
end

local resolver = luau.require_resolver()

local function extract_identifier_and_path(line: string, debug_path: string): (err.Result<string?>, string?)
    -- identifier = require("...") or require '...' or require "..."
    local identifier, requested_path = string.match(line, '([%w%.]+)%s*=%s*require%s*[%(%s]*["\'](.-)["\'][%)]?')
    if identifier and requested_path then
        return identifier, requested_path
    end

    -- require("...") or require '...' or require "@alias/path"
    requested_path = string.match(line, 'require%s*[%(%s]*["\'](.-)["\'][%)]?')
    if requested_path then
        return nil, requested_path
    end

    -- bad dynamic require: require(var)
    local dynamic_arg = string.match(line, 'require%s*%(%s*([%w_%.]+)%s*%)')
    if dynamic_arg then
        return err.message(`detected dynamic require at {debug_path}; only static requires may be bundled. offending line: {line}`), nil
    end

    return nil, nil
end

--[=[
    Check if the line is a comment.

    - If the line starts a new long comment, returns the new long comment prefix `(true, string)`.
    - If the line is within a long comment, returns the passed comment prefix `(true, string)`.
    - If the line is a single line comment, returns `(true, nil)`

    Usage

    ```luau
    local comment_prefix: string? = nil
    for l, line in readlines(f) do
        local is_comment, prefix = comment(line)
        comment_prefix = prefix
        if is_comment then
            continue
        end
    end
    ```
]=]
local function comment(line: string, current_comment_prefix: string?): (
    --- whether or not the current line is a comment
    boolean,
    --- the new or existing prefix if this line starts or is within a long comment; 
    --- nil if it ends a long comment, is a single line comment, isn't a comment, etc.
    string?
)
    if current_comment_prefix then
        local equal_signs = string.match(current_comment_prefix, "%-%-%[(=*)%[") :: string
        local closing = "]" .. equal_signs .. "]"
        if string.find(line, closing, 1, true) then
            -- ends long comment
            return true, nil
        else
            -- line within long comment
            return true, current_comment_prefix
        end
    end

    local new_comment_prefix = string.match(line, "(%-%-%[=*%[)")
    if new_comment_prefix then
        -- check if comment closed on same line as it opened
        local equal_signs = string.match(new_comment_prefix, "%-%-%[(=*)%[") :: string
        local closing = "]" .. equal_signs .. "]"

        if string.find(line, closing, 1, true) then
            -- starts and ends on the same line
            return true, nil
        else
            -- starts long comment
            return true, new_comment_prefix
        end
    end

    if str.startswith(str.trimfront(line), "--") then
        -- single line comment
        return true, nil
    end

    return false, nil
end

local anon_identifier = 0

local seen: { [string]: string? } = {}
local seen_stdlibs: { [string]: boolean? } = {}

local function traverse(path: string, depth: number?): err.Result<{string}>
    depth = depth or 0

    if depth > 333 then
        return err.message(`ahhh!!! detected circular require of '{path}' !! not allowed!`)
    end

    if not fs.path.exists(path) then
        return err.message(`cannot transform path '{path}' because it doesn't exist on the filesystem`)
    end

    local transformed: { string } = {}

    local is_comment = false
    local comment_prefix: string? = nil
    
    for line_number, line in fs.readlines(path) do
        is_comment, comment_prefix = comment(line, comment_prefix)
        if is_comment then
            continue
        end

        local identifier, requested_require = extract_identifier_and_path(line, `{path}:{line_number}`)
        if typeof(identifier) == "error" then
            return identifier
        end

        -- if it's a normal line of code w/out any imports, add it and keep going
        if requested_require == nil then
            table.insert(transformed, line)
            continue
        end

        -- if it's requiring a seal stdlib, check if we've already required it, and only add the line if we haven't yet
        if str.starts(requested_require, "@std", "@interop", "@internal") then
            if not seen_stdlibs[requested_require] then
                seen_stdlibs[requested_require] = true
                table.insert(transformed, line)
            end
            continue
        end

        -- find out where this require resolves to
        local resolved = resolver.resolve(requested_require, path)
        if resolved.err then
            return err.message(`unable to resolve require '{requested_require}' at {path}:{line_number}: {resolved.err}`)
        end

        -- if we've already seen this require (and therefore assigned an identifier to it)
        -- then just replace the require with the identifier name, add it to transformed,
        -- and continue to the next line

        local seen_identifier = seen[resolved.path]
        if seen_identifier then
            -- replace the require call with the identifier name
            local replace_parens_pattern = 'require%s*%(%s*["\']' .. requested_require .. '["\']%s*%)'
            line = string.gsub(line, replace_parens_pattern, seen_identifier)
    
            -- in case mfs use require "@myalias/idk" without parens
            local replace_string_call_pattern = 'require%s*["\']' .. requested_require .. '["\']'
            line = string.gsub(line, replace_string_call_pattern, seen_identifier)

            table.insert(transformed, line)
            continue
        end

        -- if we get here, we haven't seen this require path yet
        -- we need to inline the require in front of this line and transform it

        -- but first we need to know what the module's called; 
        -- usually identifiers are captured in local identifier = require("./some/path") form,
        -- but not all requires are local bindings.
        if identifier == nil then
            identifier = "resolved_require_" .. anon_identifier
            anon_identifier += 1         
        else
            -- captured identifiers could have dots (csv.select = require("@self/csv")) so we need to normalize
            identifier = string.gsub(identifier, "%.", "_")
            identifier = "resolved_require_" .. identifier
        end

        -- mark this dependency as seen and give it an identifier
        seen[resolved.path] = identifier

        -- now we need to recurse to get the lines of the required file,
        -- replace the `return` closest to the end of the required file w/ a local binding
        -- and then put those lines from the dependency in front of this line of code (and transform this loc)

        local dependency_lines = traverse(resolved.path, depth + 1)
        if typeof(dependency_lines) == "error" then
            return dependency_lines
        else
            dependency_lines = (dependency_lines :: any) :: { string } -- LUAU FIXME: wtf is going on with dependency_lines typestate?
        end

        for i = #dependency_lines, 1, -1 do -- go backwards
            local maybe_return_line = dependency_lines[i]
            if string.match(maybe_return_line, "^%s*return%s+") then
                -- modify dependency_lines in place to fix last return
                dependency_lines[i] = string.gsub(maybe_return_line, "^%s*return%s+", `local {identifier} = `)
                break
            end
        end

        -- and simply add all dependency lines to transformed
        table.insert(transformed, `-- START {resolved.path}`)
        for _, dependency_line in dependency_lines do
            table.insert(transformed, dependency_line)
        end

        -- replace the require call with the identifier name
        local replace_parens_pattern = 'require%s*%(%s*["\']' .. requested_require .. '["\']%s*%)'
        local replace_string_call_pattern = 'require%s*["\']' .. requested_require .. '["\']'
        line = string.gsub(line, replace_parens_pattern, identifier)
        line = string.gsub(line, replace_string_call_pattern, identifier)

        -- add finally, put the original (transformed) line behind its dependency lines!
        table.insert(transformed, line)
    end

    return transformed
end

local function bundle(project_path: string): err.Result<string>
    local entry_path = get_entry_path(project_path)
    if typeof(entry_path) == "error" then
        return entry_path
    end

    local bundled_lines = traverse(entry_path)
    if typeof(bundled_lines) == "error" then
        return bundled_lines
    end

    table.insert(bundled_lines, 1, `--!nolint LocalShadow`)
    table.insert(bundled_lines, 1, `--!optimize 2`)

    local bundled = table.concat(bundled_lines, "\n")

    return bundled
end

return bundle
