local semver = require("@std/semver")

local function lessthan()
    local s1 = semver.from("0.0.1")
    local s2 = semver.from("0.0.2")
    assert(s2 > s1, "semver s2 should be greater than s1")
end

lessthan()

local function rclessthan()
    local sbase = semver.from("0.0.2")
    local s1 = semver.from("0.0.2-rc.1")
    local s2 = semver.from("0.0.2-rc.2")

    assert(s2 > s1, "semver rc.2 should be greater than semver rc.1")
    assert(sbase > s2 and sbase > s1, "0.0.2 should be greater than any of its release candidates")
end

rclessthan()

local function le()
    local s1 = semver.from("0.2.1")
    local s12 = semver.from("0.2.1")
    assert(s1 <= s12, "should be <= s12")
    local s2 = semver.from("0.2.2")
end

le()

local function eq()
    local s1 = semver.from("0.0.2")
    local s2 = semver.from("0.0.2")
    assert(s1 == s2, "0.0.2s should be equal")
end

eq()

local function display()
    local s1 = semver.from("0.0.2")
    assert(`{s1}` == "0.0.2", "should be displayed properly as string interpolation calls tostring")
    local s2 = semver.from("0.2.3-rc.2+meow123")
    assert(`{s2}` == "0.2.3-rc.2+meow123", "more complicated display should work")
end

display()

local function satisfiesupto()
    local s1 = semver.from("0.1.0")
    assert(s1:satisfies("^0.1.0"), "should satisfy itself")
    local s2 = semver.from("1.0.0")
    assert(not s2:satisfies("^0.1.0"), "should not satisfy major version")
    assert(not semver.from("0.2.1"):satisfies("^0.1.0"), "0.2.1 should not satisfy ^0.1.0")
    assert(semver.from("0.1.2"):satisfies("^0.1.0"), "0.1.2 satisfies ^0.1.0")
    assert(semver.from("1.2.1"):satisfies("^1.0.0"), "1.2.1 should satisfy ^1.0.0")
end

satisfiesupto()
