--[=[
	Features ergonomic methods like `str.startwith`, `str.trimfront/trimback`, etc.

	This library features utf-8-aware string handling, including easy access to splitting utf-8 strings,
	iterating over the graphemes of a string, etc.

	Unlike many seal standard libraries, inputs to `str` library functions don't necessarily have
	to be valid utf-8 encoded strings.
]=]
local str = {}

--- check if a string starts with `prefix`
function str.startswith(s: string, prefix: string): boolean
	return nil :: any
end

--- check if a string ends with `suffix`
function str.endswith(s: string, suffix: string): boolean
	-- return string.sub(s, -#suffix) == suffix
	return nil :: any
end

--- trims any of the provided strings/characters from the front of the string `s`
---
--- if no strings provided as ..., `str.trimfront` will trim whitespace (" ", "\n", etc.)
function str.trimfront(s: string, ...: string): string
	-- local patterns = {...}
	-- if #patterns == 0 then
	-- 	s = string.gsub(s, "^%s*", "")
	-- else
	-- 	for _, pattern in patterns do
	-- 		while string.find(s, "^" .. pattern) do
	-- 			s = string.gsub(s, "^" .. pattern, "")
	-- 		end
	-- 	end
	-- end

	-- return s
	return nil :: any
end

--- trims any of the provided strings/characters/patterns from the back of the string `s`
---
--- if no strings provided as ..., `str.trimback` will trim whitespace (" ", "\n", etc.)
function str.trimback(s: string, ...: string): string
	-- local patterns = {...}
	-- if #patterns == 0 then
	-- 	s = string.gsub(s, "%s*$", "")
	-- else
	-- 	for _, pattern in patterns do
	-- 		while string.find(s, pattern .. "$") do
	-- 			s = string.gsub(s, pattern .. "$", "")
	-- 		end
	-- 	end
	-- end

	-- return s
	return nil :: any
end

--- trims one or many strings/characters/patterns from both front and back of string `s`
---
--- if no strings provided to `...`, then default is whitespace
function str.trim(s: string, ...: string): string
	-- return str.trimback(str.trimfront(s, ...), ...)
	return nil :: any
end

--- splits `s` by newlines, correctly handling carriage returns, trimming trailing whitespace,
--- without an extra empty string, etc.
function str.splitlines(s: string, trim_trailing_whitespace: boolean?): { string }
	-- trim_trailing_whitespace = trim_trailing_whitespace ~= false -- handle true and nil
	-- local lines = {}
	-- for line in string.gmatch(s, "[^\r\n]+") do
	--     if trim_trailing_whitespace then
	--         line = string.gsub(line, "%s*$", "")
	--     end
	--     table.insert(lines, line)
	-- end
	-- return lines
	return nil :: any
end

--- returns the utf-8 length if `s` is utf-8 or the regular string length #
function str.len(s: string): number
	-- local utf8_len = utf8.len(s)
	-- if utf8_len then
	-- 	return utf8_len
	-- else
	-- 	return #s
	-- end
	return nil :: any
end

--[=[
    `str.width` estimates the number of monospace space characters required to correctly format/pad a utf8-encoded string.

    ## Handles (or attempts to):
    - **ASCII** characters and strings.
    - Adjusts for **CJK (Chinese, Japanese, and Korean) characters**, which often take up double width.
    - Accounts for **emoji width**, ensuring proper alignment in terminal/monospace output.
    
    ## Simple usage:
    ```luau
    print(str.width("Hello")) -- 5
    print(str.width("田中良")) -- 6 (each character takes 2 spaces)
    print(str.width("🔥🎉")) -- 4 (each emoji takes 2 spaces)
    ```

	## Actual example:
	```luau
	local students = {
		{ name = "Alex Johnson", score = 95 },
		{ name = "田中良", score = 88 },
		{ name = "🔥🎉 Emily Carter", score = 92 },
		{ name = "Nadiya Kovalenko", score = 85 },
	}

	-- Calculate column widths dynamically using `str.width`
	local max_name_width = 0
	for _, student in students do
		max_name_width = math.max(max_name_width, str.width(student.name))
	end

	-- Print formatted table
	print("Name" .. string.rep(" ", max_name_width - str.width("Name")) .. " | Score")
	print(string.rep("-", max_name_width) .. "-|------")

	for _, student in students do
		print(
			student.name
			.. string.rep(" ", max_name_width - str.width(student.name)) 
			.. " | " .. student.score
		)
	end
	```
]=]
function str.width(s: string): number
	-- local length = str.len(s)

	-- for _, grapheme in str.graphemes(s) do
	--     local codepoint = utf8.codepoint(grapheme)

	--     -- Detect **CJK wide characters**
	--     if (codepoint >= 0x4E00 and codepoint <= 0x9FFF) -- Common CJK
	--     or (codepoint >= 0x3400 and codepoint <= 0x4DBF) -- CJK Extension A
	--     or (codepoint >= 0x20000 and codepoint <= 0x2A6DF) -- CJK Extension B
	--     or (codepoint >= 0x2A700 and codepoint <= 0x2B73F) -- CJK Extension C
	--     or (codepoint >= 0x2B740 and codepoint <= 0x2B81F) -- CJK Extension D
	--     or (codepoint >= 0x2B820 and codepoint <= 0x2CEAF) -- CJK Extension E
	--     or (codepoint >= 0xF900 and codepoint <= 0xFAFF) -- CJK Compatibility Ideographs
	--     then
	--         length += 1 -- Treat CJK characters as wide
	--     end

	--     -- Detect **emoji width**
	--     if codepoint >= 0x1F300 and codepoint <= 0x1FAFF then
	--         length += 1 -- Emoji takes extra width
	--     end
	-- end

	-- return length
	return nil :: any
end

--- left pads `s` to make it at least `width` characters long, using `pad` as the padding character.
function str.leftpad(s: string, width: number, pad: string?): string
	-- pad = pad or " " -- default to space if no padding character is provided
	-- local padding_needed = width - #s
	-- if padding_needed > 0 then
	--     return string.rep(pad, padding_needed) .. s
	-- end
	-- return s
	return nil :: any
end

--- escapes special characters like `\n`, `\t`, `\\` for easier debugging
function str.escape(s: string): string
	-- return (string.gsub(s, "[\\\n\t\r]", {
	--     ["\\"] = "\\\\",
	--     ["\n"] = "\\n",
	--     ["\t"] = "\\t",
	--     ["\r"] = "\\r"
	-- }))
	return nil :: any
end

--- reverts `str.escape`
function str.unescape(s: string): string
	-- local unescape_map = {
	--     ["\\n"] = "\n",
	--     ["\\t"] = "\t",
	--     ["\\r"] = "\r",
	--     ["\\\\"] = "\\"
	-- }
	-- return (string.gsub(s, "\\(.)", unescape_map))
	return nil :: any
end

--- alias for string.sub
function str.slice(s: string, first: number, final: number)
	-- return string.sub(s, first, final)
	return nil :: any
end

--- indents multiline string `count` characters; lines separated by `sep` (default "\n")
function str.indent(s: string, whitespace_type: "Tabs" | "Spaces", count: number, sep: ("\n" | "\r\n")?): string
	-- local indent = if whitespace_type == "Tabs" then string.rep("\t", count) else string.rep(" ", count)
	-- local sep = sep or "\n" -- allows users to pass in \r\n or w/e if needed

	-- local result = {}
	-- local lines = str.splitlines(s)
	-- for _, line in lines do
	--     table.insert(result, indent .. line)
	-- end

	-- return table.concat(result, sep)
	return nil :: any
end

--- unindents multiline string by `count` characters; lines separated by `sep` (default "\n")
function str.unindent(s: string, whitespace_type: "Tabs" | "Spaces", count: number, sep: ("\n" | "\r\n")?): string
	-- local indent = if whitespace_type == "Tabs" then string.rep("\t", count) else string.rep(" ", count)
	-- local sep = sep or "\n"

	-- local result = {}
	-- for _, line in str.splitlines(s) do
	--     if str.startswith(line, indent) then
	--         local trimmed_line = string.gsub(line, "^" .. indent, "")
	--         table.insert(result, trimmed_line)
	--     else
	--         table.insert(result, line)
	--     end
	-- end

	-- return table.concat(result, sep)
	return nil :: any
end

local internal = (require)("@std/str_internal")

--[=[
	`str.split` is an improvement on luau's `string.split` in that it can split by multiple different strings (not just one single character)
	at the same time and that the splitting is fully unicode grapheme aware.

	If no separators are passed, `str.split` splits the string by graphemes (human-readable unicode characters);
	otherwise, splitting is performed by the Aho-Corasick algorithm, which allows for efficient string splitting
	with multiple separator strings.

	## Usage
	```luau
	local chars = str.split("seals 🦭 ") 
	--> { "s", "e", "a", "l", "s", " ", "🦭", " "  }
	local words = str.split("seals 🦭 say hi", " ") 
	--> { "seals", "🦭", "say", "hi" }
	local omit_hi = str.split("seals 🦭 say hi", " ", "hi")
	--> { "seals", "🦭", "say" }
	```

	### Notes
	- Like with Luau's `string.split`, passing an empty separator string (`""`) to `str.split` will split the string by bytes instead of graphemes.
	- splits that result in an empty string are not included in the returned array.
	- `str.split` does not allow for overlapping splits when split with multiple separators.
	- Separators are evaluated in left-to-right order, meaning that separators in front have higher priority than those in the back.


	### Edge cases:
	- Sometimes simple characters directly to the right of emojis don't render when printed (example `print[["🦭"]]`)
	- Some Hindi graphemes (like हा) don't render properly in terminals :(
]=]
str.split = internal.split :: (s: string, ...string) -> { string }

--[=[
	Splits string `s` *around* one or more separator strings, keeping the separators in the final result.
	This is especially useful for parsing and tokenizing text.

	`str.splitar` otherwise follows the same semantics as `str.split`, including splitting by unicode graphemes when zero separators are provided.

	Separators are evaluated in left-to-right order, meaning that separators in front have higher priority than those in the back.

	Like `str.split`, `str.splitar` is fully unicode grapheme-aware and can operate on full strings (instead of just characters).

	## Usage
	
	```luau
	local line = `function Cat.meow(name: string, age: number)`
	local tokens = str.splitar(line, " ", ".", "(", ":", ",", ")")
	--> { "function", " ", "Cat", ".", "meow", "(", "name", ":", " ", "string", ",", " ", "age", ":", " ", "number", ")"}
	```
]=]
str.splitar = internal.splitar :: (s: string, ...string) -> { string }

--[=[
	Iterate over the human-readable characters (graphemes) of a string

	This function counts by 'characters', whereas `str.graphemes` provides byte indices for `string.sub`/`str.slice`
]=]
function str.chars(s: string): (...any) -> (number, string)
	-- local current_index = 0
	-- local next_grapheme = internal.graphemes(s)
	-- return function()
	-- 	local bytes, grapheme = next_grapheme()
	-- 	if bytes ~= nil then
	-- 		current_index += 1
	-- 		return current_index, grapheme
	-- 	else
	-- 		return nil, nil :: any
	-- 	end
	-- end
	return nil :: any
end

--[=[
	Iterate over the utf-8 graphemes of `s` with indices useful for `str.slice` or `string.sub`

	## Usage
	```luau
	local str = require("@std/str")

	local utf8_string = "सील hi i am a seal 🦭"
	for offset, grapheme in str.graphemes(utf8_string) do
		print(`found '{grapheme}' starting at {offset} and ending at {offset + #grapheme}`)
	end
	```

	### Edge cases:
	- Sometimes simple characters directly to the right of emojis don't render when printed (example "🦭")
	- Some Hindi graphemes (like हा) don't render properly in terminals :(
]=]
str.graphemes = internal.graphemes :: (s: string) -> (...any) -> (number, string)

return str
