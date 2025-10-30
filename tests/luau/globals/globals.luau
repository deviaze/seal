--!nocheck

-- does _G exist and can we do things to it
assert(_G ~= nil, "why doesn't _G exist??")
_G.hmm = "_hmm"
assert(_G.hmm == "_hmm", "can't assign simple variable to _G")

-- do regular globals work?
meow = 2
assert(meow == 2, "assignment to globals borked")

local reserved = (require :: any)("@internal/reserved_aliases")
print(reserved)

-- -- can we access that global through _G?
-- print(`access meow global thru _G: {_G.meow}`)

-- -- can we get the REAL globals table with print and warn and stuff thru _G?
-- print("can we get the REAL globals?")
-- local real_globals = getmetatable(_G)
-- -- print(real_globals)
-- local n = 0
-- print(_G.print)
