local fs = require("@std/fs")

local resolver_src: string = "" do
	local resolver_dot_luau_path = fs.path.join(".", "src", "require", "resolver.luau")
	resolver_src = fs.readfile(resolver_dot_luau_path)
end

-- hope for/assume this works; if not well test should fail lol
local resolver = require("./../../../../src/require/resolver")

-- the regular require tests should handle regular cases

local datadir = fs.dir.from("./tests/data")

local luaurc_contents = [[
	{
		"aliases": {
			"libraries": "./libraries/"
		}
	}
]]

local testtree = fs.tree()
	:with_file(".luaurc", luaurc_contents)
	:with_tree("src", fs.tree()
		:with_file("main.luau", "return require('./hi')")
		:with_file("hi.luau", "return 'hi'")
		:with_tree("biblicallyaccuratemodule", fs.tree()
			:with_file("init.luau", "return 'bib'")
			:with_file("bibinternal.luau", "return 'secret bib internals'")
		)
	)
	:with_tree("libraries", fs.tree()
		:with_file("leaf.luau", "return 'leaf'")
	)

local resolver_project_path = datadir:join("resolvertestproject")

if fs.path.exists(resolver_project_path) then
	fs.removetree(resolver_project_path)
end

fs.writetree(resolver_project_path, testtree)
local proj_dir = fs.dir.from(resolver_project_path)

local abs_proj_path = fs.path.canonicalize(resolver_project_path)

local function make_debug_name(p: string): string
	return `[string "{p}"]`
end

local main_luau_debugname = make_debug_name(fs.path.join(abs_proj_path, "src", "main.luau"))

local function simplerelativeresolve()
	local hi_result = resolver.resolve(
		"./hi", 
		main_luau_debugname, 
		false
	) :: any
	assert(
		hi_result.path == fs.path.join(abs_proj_path, "src", "hi.luau"), 
		"simple relative resolve not resolving?"
	)
end

simplerelativeresolve()

local bibpath = fs.path.join(abs_proj_path, "src", "biblicallyaccuratemodule")
local bibdebugname = make_debug_name(fs.path.join(bibpath, "init.luau"))

local function oldsemantics()
	local bib_result = resolver.resolve(
		"./bibinternal",
		bibdebugname,
		false
	) :: any
	assert(
		bib_result.path == fs.path.join(bibpath, "bibinternal.luau"),
		"old semantics sholud resolve bibinternal"
	)
end

oldsemantics()

local function selfsemantics()
	local self_result = resolver.resolve(
		"@self/bibinternal",
		bibdebugname
	) :: any
	assert(
		self_result.path == fs.path.join(bibpath, "bibinternal.luau"),
		"@self alias should resolve to bibinternal.luau"
	)
end

selfsemantics()

local function newsemantics()
	local hi_result = resolver.resolve(
		"./hi",
		bibdebugname
	) :: any
	assert(
		hi_result.path == fs.path.join(abs_proj_path, "src", "hi.luau"),
		"seal should fallback to new semantics (thereby finding proj/src/hi.luau)"
	)
end

newsemantics()

local function aliases()
	local leaf_res = resolver.resolve(
		"@libraries/leaf",
		main_luau_debugname
	) :: any
	assert(
		leaf_res.path == fs.path.join(abs_proj_path, "libraries", "leaf.luau"),
		"requiring local alias should work even if more aliases are defined upstream"
	)

	local opv_result = resolver.resolve(
		"@output_formatter",
		bibdebugname
	) :: any
	assert(
		opv_result.path == fs.path.join(fs.path.cwd(), "src", "std_io", "output_formatter.luau"),
		"requiring aliases defined upwards (not closest .luaurc) should also resolve"
	)
end

aliases()

proj_dir:remove()
