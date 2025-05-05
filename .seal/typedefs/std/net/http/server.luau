local server = {}

type StatusCode =
	| "200 OK"
	| "201 Created"
	| "204 No Content"
	| "301 Moved Permanently"
	| "302 Found"
	| "304 Not Modified"
	| "307 Temporary Redirect"
	| "308 Permanent Redirect"
	| "400 Bad Request"
	| "401 Unauthorized"
	| "403 Forbidden"
	| "404 Not Found"
	| "405 Method Not Allowed"
	| "409 Conflict"
	| "410 Gone"
	| "412 Precondition Failed"
	| "415 Unsupported Media Type"
	| "429 Too Many Requests"
	| "500 Internal Server Error"
	| "501 Not Implemented"
	| "502 Bad Gateway"
	| "503 Service Unavailable"
	| "504 Gateway Timeout"
	| "505 HTTP Version Not Supported"

type ContentType = 
	| "Text"
	| "HTML"
	| "JSON"
	| "XML"
	| "CSS"
	| "JavaScript"
	| "Binary"
	| string

export type ServeRequest = {
	peer_address: string,
	method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE",
	path: string,
	headers: {
		[string]: string,
	},
	raw_text: string,
	body: string,
}
export type ServeResponse = {
	status_code: StatusCode,
	content_type: ContentType,
	body: string,
	headers: {
		[string]: string,
	}?,
	cookies: {
		[string]: string,
	}?,
	http_version: string?,
	reason_phrase: string?,
	redirect_url: string?
}
	
export type ServeConfig = {
	address: string,
	port: string | number,
	handler: (ServeRequest) -> ServeResponse,
}

function server.serve(config: ServeConfig)
	
end

return server