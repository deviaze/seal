local fs = require("@std/fs")
local output = require("@std/io/output")

local _dp = output["debug-print"]

local server_views = fs.dir.from("./tests/data/server-views")
local index_file = server_views:expect_file("index.html")

local read_normally = index_file:read()
local split_lines = read_normally:split("\n")

for line_number, line in index_file:readlines() do
	if line_number == 1 then
		assert(line:match("<!DOCTYPE html>"), "line 1 doctype doesnt match")
	end
	local normalized_line = split_lines[line_number]:gsub("\r", "") -- windows \r
	assert(normalized_line == line, `line {line_number} doesn't match`)
end

local funny_story_path = fs.path.join(".", "tests", "data", "funny_story.txt")
local story_normally = fs.readfile(funny_story_path):split("\n")
for line_number, line in fs.readlines(funny_story_path) do
	local normalized_line = story_normally[line_number]:gsub("\r", "")
	assert(normalized_line == line, `line {line_number} not accurate`)
end
