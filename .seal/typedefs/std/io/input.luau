--- input handling lib
local input = {}

--- gets input with an optional `raw_prompt` to display before getting said input.
function input.get(raw_prompt: string): string
	return nil :: any
end

-- type input = {
-- 	--- gets input with an optional `raw_prompt` to display before getting said input.
-- 	get: (raw_prompt: string?) -> string, 
-- }

type input = typeof(input)

return input :: input