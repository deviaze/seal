-- note! although this library is implemented in luau, modifying its typedefs won't
-- modify it in seal!

local str = require("@std/str")

--[=[
### semver = semantic versioning

This implementation roughly follows the described spec at https://semver.org.

Basically, a semver consists of 3 components: major, minor, and patch, with an optional
`-rc.<number>` release candidate suffix or `+buildnumber` build suffix:

**MAJOR.MINOR.PATCH**

Examples:
- `1.0.0` - unheard of in the rust or luau community
- `0.0.1-rc.1`
- `0.2.0+build231`

To adhere to semver, you should follow these rules:
- Increment the **major** version when you make incompatible API changes, or the **minor** version
    if your project is < `1.0.0`.
- Increment the **minor** version when you add functionality in a backwards-compatible manner.
- Increment the **patch** version when you make backwards-compatible bug fixes.
- Use a **release candidate** suffix (`-rc.<number>`) to indicate pre-release versions that are not yet stable.
- Use a **build** suffix (`+build123`) to attach metadata that does not affect version precedence.
- Pre-release versions are ordered before their corresponding release: `1.0.0-rc.1 < 1.0.0`.
- Build metadata is ignored when comparing versions: `1.0.0+abc == 1.0.0+xyz`.

## Usage

```luau
local semver = require("@std/semver")
local first_version = semver.from("0.0.1")
local second_version = semver.from("0.0.2")

first_version:satisfies("^0.0.1") --> true
second_version:satisfies("^0.0.1") --> true
assert(first_version < second_version)
```
]=]
local semver = {}
semver.__index = semver

type SemverFields = {
    major: number,
    minor: number,
    patch: number,
    metadata: {
        release_candidate: {
            tag: string,
            ver: number,
        }?,
        build: string?,
    }
}

type SemverImpl = typeof(semver)

export type Semver = setmetatable<SemverFields, SemverImpl>

function semver.default(): Semver
    local fields: SemverFields = {
        major = 0,
        minor = 1,
        patch = 0,
        metadata = {},
    }
    return setmetatable(fields, semver)
end

function semver.from(s: string): Semver
    local major, minor, patch = string.match(s, "^(%d+)%.(%d+)%.(%d+)")
    if not major or not minor or not patch then
        error("Invalid semver string: " .. s)
    else
        major = tonumber(major)
        minor = tonumber(minor)
        patch = tonumber(patch)
        if not major or not minor or not patch then
            error(`Invalid semver string: {s}`)
        end
    end

    local metadata = {}

    -- Extract release candidate (e.g., -rc.1)
    local rc_name, rc_version = string.match(s, "%-(%a+)%.(%d+)")
    if rc_name and rc_version then
        metadata.release_candidate = {
            tag = rc_name,
            ver = tonumber(rc_version) :: number,
        }
    end

    -- Extract build metadata (e.g., +build123)
    local build = string.match(s, "%+([%w%.%-]+)")
    if build then
        metadata.build = build
    end

    assert(
        typeof(major) == "number" 
        and typeof(minor) == "number" 
        and typeof(patch) == "number",
        `Unexpected major/minor/patch ({major}, {minor}, {patch})`
    )

    local fields: SemverFields = {
        major = major,
        minor = minor,
        patch = patch,
        metadata = metadata :: any, -- LUAU FIXME: not optional fields not qualifying as optionals
    }

    return setmetatable(fields, semver)
end

local semver_ranges = {
    ["^"] = "AND UP",
    ["=="] = "EXACTLY EQUAL",
    ["<="] = "LESS THAN OR EQUAL",
    ["<"] = "LESS THAN",
    [">"] = "GREATER THAN NOT INCLUDING",
}

type SemverVals = 
    | "AND UP" 
    | "EXACTLY EQUAL" 
    | "LESS THAN OR EQUAL" 
    | "LESS THAN" 
    | "GREATER THAN NOT INCLUDING"

--[=[
    Returns true if the `self` is compatible with (within the range of) semver_range.

    `semver_range` supports the following syntaxes:
    - `^` like `^0.1.0`, satisfied by any semvers greater than or equal to `0.1.0` but less than `0.2.0`,
    - `==` like `==0.1.0` for exact matches,
    - `<=` like `<=1.0.0` for upper bounds that are not necessarily equivalent to ^,
    - `>` like `<1.0.1` for lower bounds (exclusive),
    - Defaults to `^` when no operator provided (`0.2.1` defaults to `^0.2.1`),
    - Multiple constraints can be space-separated, e.g. `>=1.2.3 <2.0.0`, which all must be satisfied.

    Note that release candidates (rc.<number>) are ordered before full releases, therefore
    `0.2.0-rc.1` < `0.2.0`.
    
    ## Usage

    ```luau
    local semver = require("@std/semver")

    local some_version = semver.from(require("./config.luau").version)
    if some_version:satisfies("^0.1.0") then
        print("compatible version!")
    else
        print("incompatible version :(")
    end
    ```
]=]
function semver.satisfies(self: Semver, semver_range: string): boolean
    local function get_specific_range(part: string): (SemverVals, string)
        for k, _ in semver_ranges :: { [string]: string } do
            if str.startswith(part, k) then
                local op = (semver_ranges :: any)[k] :: SemverVals
                local version = str.trimfront(part, k)
                return op, version
            end
        end
        return "AND UP", part
    end

    local parts = string.split(semver_range, " ")
    for _, part in parts do
        local op, version_str = get_specific_range(part)
        local other = semver.from(version_str)

        local satisfies = false

        if op == "AND UP" then
            local upper_bound = 
                if other.major > 0 then
                    semver.from(`{other.major + 1}.0.0`)
                elseif other.minor > 0 then
                    semver.from(`0.{other.minor + 1}.0`)
                else
                    semver.from(`0.0.{other.patch + 1}`)
            satisfies = self >= other and self < upper_bound
        elseif op == "EXACTLY EQUAL" then
            satisfies = (self :: any) == other -- LUAU FIXME
        elseif op == "LESS THAN" then
            satisfies = self < other
        elseif op == "GREATER THAN NOT INCLUDING" then
            satisfies = self > other
        else
            satisfies = false
        end

        if not satisfies then
            return false
        end
    end

    return true
end

function semver.__eq(self: Semver, other: Semver): boolean
    if 
        self.major ~= other.major or
        self.minor ~= other.minor or
        self.patch ~= other.patch 
    then
        return false
    end

    local a_rc = self.metadata.release_candidate
    local b_rc = other.metadata.release_candidate

    if (a_rc == nil) ~= (b_rc == nil) then
        return false
    elseif a_rc and b_rc then
        if a_rc.tag ~= b_rc.tag or a_rc.ver ~= b_rc.ver then
            return false
        end
    end

    if self.metadata.build ~= other.metadata.build then
        return false
    end

    return true
end

function semver.__lt(self: Semver, other: Semver): boolean
    if self.major ~= other.major then
        return self.major < other.major
    end
    if self.minor ~= other.minor then
        return self.minor < other.minor
    end
    if self.patch ~= other.patch then
        return self.patch < other.patch
    end

    local a_rc = self.metadata.release_candidate
    local b_rc = other.metadata.release_candidate

    -- Pre-release versions are lower than normal versions
    if a_rc == nil and b_rc ~= nil then
        return false
    elseif a_rc ~= nil and b_rc == nil then
        return true
    elseif a_rc and b_rc then
        if a_rc.tag ~= b_rc.tag then
            return a_rc.tag < b_rc.tag -- lexicographic comparison
        end
        return a_rc.ver < b_rc.ver -- numeric comparison
    end

    -- build metadata is ignored for ordering
    return false
end

function semver.__le(self: Semver, other: Semver): boolean
    if self.major ~= other.major then
        return self.major <= other.major
    end
    if self.minor ~= other.minor then
        return self.minor <= other.minor
    end
    if self.patch ~= other.patch then
        return self.patch <= other.patch
    end

    local a_rc = self.metadata.release_candidate
    local b_rc = other.metadata.release_candidate

    -- Pre-release versions are lower than normal versions
    if a_rc == nil and b_rc ~= nil then
        return false
    elseif a_rc ~= nil and b_rc == nil then
        return true
    elseif a_rc and b_rc then
        if a_rc.tag ~= b_rc.tag then
            return a_rc.tag <= b_rc.tag -- lexicographic comparison
        end
        return a_rc.ver <= b_rc.ver -- numeric comparison
    end

    -- Build metadata is ignored for ordering
    return true
end

function semver.__tostring(self: Semver): string
    local result = `{self.major}.{self.minor}.{self.patch}`
    if self.metadata.release_candidate then
        result ..= `-{self.metadata.release_candidate.tag}.{self.metadata.release_candidate.ver}`
    end
    if self.metadata.build then
        result ..= `+{self.metadata.build}`
    end
    return result
end

return semver