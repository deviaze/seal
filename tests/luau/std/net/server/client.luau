-- test client to use for net.http.serve stuff

local http = require("@std/net/http")

local response = http.post {
	url = "http://localhost:4242/some-post",
	body = {
		hello = true,
	}
}:unwrap_json()

assert(response.recvbody == [[{"hello":true}]], "response.recvbody doesn't match")

local invalid_request = http.post {
	url = "https://localhost:4242/some-post",
	body = {
		hello = true
	},
}

print(invalid_request)