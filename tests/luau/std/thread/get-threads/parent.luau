--!nonstrict
--!nolint LocalShadow
local thread = require("@std/thread")
local format = require("@std/io/format")

local urls: {string} = {
	"https://meow.com",
	"https://google.com",
	"https://en.wikipedia.org/wiki/Cat",
	"https://catfact.ninja/fact",
	"https://jsonplaceholder.typicode.com/todos/1",
}

-- get in parallel
local s_t = os.clock()

local handles: { [string]: thread.ThreadHandle } = {}
local results: { [string]: string } = {}

for _, url in urls do
	local handle = thread.spawn({
		path = "./send_request.luau",
		data = {
			url = url,
		}
	})
	handles[url] = handle
end

local start_time = os.clock()
local results_got = 0
while (os.clock() - start_time) < 20 do
	for url, handle in handles do
		local result = handle:read()
		if result then
			results[url] = result
			results_got += 1
			handles[url] = nil
		end
	end
	if results_got == #urls then
		break
	end
end

for url, handle in handles do
	handle:join()
end

print(handles)

print(`{os.clock() - s_t} seconds in parallel/synchronously with os threads :)`)

local results_by_len = {}
for url, result in results do
	results_by_len[url:gsub("https://", "")] = (
		if #result == 0 then `json len: {#format(result)} chars`
		else `html len: {#result} chars`
	)
end
print(results_by_len)