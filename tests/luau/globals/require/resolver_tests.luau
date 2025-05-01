local fs = require("@std/fs")

local resolver_src: string = "" do
	local resolver_dot_luau_path = fs.path.join(".", "src", "require", "resolver.luau")
	resolver_src = fs.readfile(resolver_dot_luau_path)
end

-- hope for/assume this works; if not well test should fail lol
local resolve = require("./../../../../src/require/resolver")

local function differentcwd()
	-- -- sanity checks
	-- local t1 = `[string "/home/deviaze/Desktop/Luau/navigator/src/main.luau"]`
	-- local r = resolve("../tests/run", t1)
	-- print(r)
	-- local r2 = resolve("@libs/cats", t1)
	-- print(r2)

	-- print(debug.info(1, "s"))
end

differentcwd()

-- print(fs.readfile("staftfa"))
