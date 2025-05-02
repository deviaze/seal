--!strict
--!nolint LocalShadow
local http = require("@std/net/http")

print(
	http.get {
		url = "https://jsonplaceholder.typicode.com/todos/1"
	}:unwrap_json()
)

print(
	http.get {
		url = "https://jsonplaceholder.typicode.com/posts",
		params = {
			userId = "1",
		}
	}:unwrap_json()
)

local response_with_status_code = http.get {
	url = "https://jsonplaceholder.typicode.com/posts/200"
}

if response_with_status_code.ok then
	assert(response_with_status_code.status_code == "404 Not Found")
end

-- attempt to call :unwrap() with missing arg
local success, result = pcall(function()
	local should_actually_error = http.get {
		url = "",
		params = {
			catname = "Taz",
		}
	}:unwrap_json()
	return should_actually_error
end)
local result = tostring(result)

assert(
	success == false
	and result:match("without default argument")
	,
	"expected to match unwrap without default arg, didnt match"
)

-- attempt to call unwrap(nil)
local x = nil
local computed = x or nil

local success, result = pcall(function()
	return http.get({
		url = "memes.com",
		body = {
			x = 2,
			y = 3,
		}
	}):unwrap_json(nil)
end)
local result = tostring(result)

assert(
	success == false
	and result:match("note: default argument provided but was nil")
	, 
	"that was supposed to break"
)