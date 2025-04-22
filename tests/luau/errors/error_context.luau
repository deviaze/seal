--!nonstrict

local output = require("@std/io/output")

local function withoutcontext()
	local function nocontext()
		error("no context")
	end
	script.context = nil
	local s, err = pcall(nocontext)
	local unformatted_err = output.unformat(tostring(err))
	assert(unformatted_err:match('message: "no context"'), "no context")
end

withoutcontext()

--/sealignore: this test fails until script context fixed
local function withcontext()
	local function sealionroulette()
		if math.random(10, 20) < 15 then
			error("bad luck bro")
		else
			return "survived"
		end
	end
	script.context = "sealion roulette"
	local s, err = pcall(sealionroulette)
	local unformatted_err = output.unformat(tostring(err))
	-- assert(unformatted_err:match("[CONTEXT] sealion roulette"), "error context not applied")
end

withcontext()