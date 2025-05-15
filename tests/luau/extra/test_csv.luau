-- TODO, actually fill out the unit tests

local str = require("@std/str")
local path = require("@std/fs/path")
local csv = require("@extra/csv")

local csv_data_path = path.join(".", "tests", "data", "csv_data")
-- just being able to read and parse these files means it handles utf8 and emojis correctly
local courses = csv.readfile(path.join(csv_data_path, "courses.csv"))
local students = csv.readfile(path.join(csv_data_path, "students.csv"))

local merged = csv.join(courses, students, "CRN", "CRN")

local function expectedcolumns()
	-- print(merged.column_names)
	assert(
		merged.column_names[1] == "CRN"
		and merged.column_names[7] == "Major",
		"first and last col names not as expected"
	)
end

expectedcolumns()

local function canweselect()
	local cs_students = csv.select(merged, {
		with_columns = { "Student Name", "Class Name" },
		where = {
			["Major"] = function(major: string)
				return if string.find(major, "Computer") then true else false
			end,
		}
	})

	local r1 = csv.rows(cs_students, 1, 1)
	assert(r1[1][1]["Student Name"] == "Alex Johnson", "r1 not alex johnson")
	
end

canweselect()

local function writeandretrieve()
	local merged_data_path = path.join(csv_data_path, "merged_data.csv")
	csv.writefile(merged_data_path, merged)
	local retr = csv.readfile(merged_data_path)
	assert(
		str.startswith(retr.rows[1][2], "Introduction"),
		"retrieved csv row 1 col 2 not start with Introduction?"
	)
end

writeandretrieve()