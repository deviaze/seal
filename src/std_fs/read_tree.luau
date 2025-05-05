-- doesn't make sense to write in Rust
local fs = require("@std/fs")

type TreeResult = {
	tree: fs.DirectoryTree
} | {
	err: string
}

local function readtree(path: string): TreeResult
	local directory_tree = {}
	for p, entry in fs.entries(path) do 
		if entry.type == "File" then
			table.insert(directory_tree, {
				type = "File",
				name = entry.name,
				content = entry:read(),
			})
		elseif entry.type == "Directory" then
			local result = readtree(p)
			if (result :: any).tree then -- LUAU FIXME: doesn't realize it's a table refinement check
				table.insert(directory_tree, {
					type = "Directory",
					name = entry.name,
					children = result.tree,
				})
			else
				return result
			end
		end
	end
	return {
		tree = directory_tree
	}
end

return readtree