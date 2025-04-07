local http = require("@std/net/http")
-- local process = require("@std/process")

local r = http.get {
	url = "meow.com"
}
local t0 = os.clock()
local catfact_result: http.HttpResponse = http.get { url = "https://catfact.ninja/fact" } 
if catfact_result.ok then
	print(catfact_result:decode())
end
print(`catfact time: {os.clock() - t0}`)

local t1 = os.clock()
local placeholder: http.HttpResponse = http.get {
	url = "https://jsonplaceholder.typicode.com/posts/1"
}
if placeholder.ok then
	print(placeholder:decode())
end
print(`jsonplaceholder time: {os.clock() - t1}`)
