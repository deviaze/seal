--!nolint LocalShadow
local request = require("@std/net/request")

local result = request {
	url = "https://jsonplaceholder.typicode.com/todos/1",
	method = "DELETE",
}
assert(result.ok, "issue deleting from placeholder endpoint")

local result = request {
	url = "https://jsonplaceholder.typicode.com/posts/1",
	method = "PATCH",
	body = {
		title = "hmmmm",
	}
}:unwrap_json()
assert(result.title == "hmmmm", "issue patching placeholder endpoint")
