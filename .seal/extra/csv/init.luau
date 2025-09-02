local fs = require("@std/fs")
local str = require("@std/str")
local output = require("@std/io/output")

local types = require("@self/types")
export type Csv = types.Csv

local row = require("@self/row")
type CsvRow = row.CsvRow
local column = require("@self/column")
type CsvColumn = column.CsvColumn

--[=[
	A simple, utf8-aware csv library with nice pretty printing capabilities and even some
	select/filter operations.

	This library is pretty barebones for now, please add expansions (and pr them in) for more features.
]=]
local csv = {}

local function parse_line(line: string): { string }
	local result: { string } = {}
	local current_word = ""
	local in_quotes = false
	local quotes_in_a_row = 0

	for _, grapheme in str.graphemes(line) do
		if grapheme == '"' then
			quotes_in_a_row += 1
			in_quotes = not in_quotes
		elseif grapheme == "," and not in_quotes then
			table.insert(result, current_word)
			current_word = ""
			quotes_in_a_row = 0
			continue
		else
			quotes_in_a_row = 0
		end

		if quotes_in_a_row == 2 and current_word == "" then
			continue
		end

		if quotes_in_a_row == 2 then
			current_word ..= '"'
			quotes_in_a_row = 0
		end

		if grapheme ~= '"' then
			current_word ..= grapheme
		end
	end

	-- last column doesn't have trailing ","
	table.insert(result, current_word)

	return result
end

--- read csv from file into a `Csv`
function csv.readfile(path: string): Csv
	local text = fs.readfile(path)
	return csv.parse(text)
end

function csv.writefile(path: string, c: Csv)
	local formatted_csv = csv.serialize(c)
	fs.writefile(path, formatted_csv)
end

--- create/parse a `Csv` from a properly formatted csv string
---
--- note that escaped quotes inside a string should be double quoted as per the csv specification
function csv.parse(raw_text: string)
	local lines = str.splitlines(raw_text)
	local first_line = table.remove(lines, 1) or error("csv empty?")

	local column_names = parse_line(first_line)
	-- stylua: ignore
	local column_map = {} do
		for column_index, name in column_names do
			column_map[name] = column_index
		end
	end

	local rows = {}
	-- stylua: ignore
	local columns = {} do
		for column_index, column_name in column_names do
			columns[column_index] = {
				column_name = column_name,
				values = {},
			}
		end
	end

	for row_index, line in lines do
		local current_row = {}
		local parsed_line = parse_line(line)
		for column_index_in_row, value in parsed_line do
			current_row[column_index_in_row] = value
			local current_column_values = columns[column_index_in_row].values
			table.insert(current_column_values, value)
		end
		table.insert(rows, current_row)
	end

	local new_csv = {
		column_names = column_names,
		column_map = column_map,
		columns = columns,
		rows = rows,
	}
	return new_csv :: Csv
end

--- shows the top `n` rows of the csv
function csv.top(c: Csv, n: number): ()
	return csv.show(c, 1, n)
end

--- show (print) rows of a csv from `start_row` to `end_row`
function csv.show(c: Csv, start_row: number, end_row: number): ()
	local col_widths = {}
	for col_index, name in c.column_names do
		local max_width = str.width(name)
		for row_index = start_row, math.min(end_row, #c.rows) do
			local row = c.rows[row_index]
			if row[col_index] then
				max_width = math.max(max_width, str.width(row[col_index]))
			end
		end
		col_widths[col_index] = max_width
	end

	-- Print header row with correct padding
	for col_index, name in c.column_names do
		output.write(name .. string.rep(" ", math.max(0, col_widths[col_index] - str.width(name))) .. "  ")
	end
	output.write("\n")

	-- Print separator line
	for col_index, _ in c.column_names do
		output.write(string.rep("-", col_widths[col_index]) .. "  ")
	end
	output.write("\n")

	-- Print rows from `from` to `to` with correct spacing
	for row_index = start_row, math.min(end_row, #c.rows) do
		local row = c.rows[row_index]
		for col_index, _ in c.column_names do
			local value = row[col_index] or ""
			output.write(value .. string.rep(" ", math.max(0, col_widths[col_index] - str.width(value))) .. "  ")
		end
		output.write("\n")
	end
end

--- sort the csv in place in alphabetical order by `column_name`
function csv.sort(c: Csv, column_name: string): ()
	local column_index = nil
	for col_index, name in c.column_names do
		if name == column_name then
			column_index = col_index
			break
		end
	end

	if not column_index then
		error("column '" .. column_name .. "' not found in csv")
	end

	table.sort(c.rows, function(a: { string }, b: { string })
		return (a[column_index] or "") < (b[column_index] or "")
	end)
end

--[=[
	`csv.join` performs an inner join on two `Csv`s based on one common column.
	
	the merged csv retains all columns from both tables, excluding duplicate join columns.

	## Notes:
	- correctly handles multiple rows in `right` that match to the same row in `left`
	- rows that can't be matched on that column between both `Csv`s are excluded from the merged result
	- the resulting csv structure maintains column order and properly maps values across matching rows.  
]=]
function csv.join(left: Csv, right: Csv, left_on: string, right_on: string): Csv
	-- find column indices for joining
	local left_index, right_index = left.column_map[left_on], right.column_map[right_on]
	if not left_index or not right_index then
		error(`Join columns not found: left_on: {left_on}, right_on: {right_on}`)
	end

	local joined_csv = {
		column_names = {},
		column_map = {},
		columns = {},
		rows = {},
	}

	-- merge column names, ensuring no duplicate join column
	local right_column_offset = #left.column_names
	for column_position, column_name in left.column_names do
		table.insert(joined_csv.column_names, column_name)
		joined_csv.column_map[column_name] = column_position
		joined_csv.columns[column_position] = { column_name = column_name, values = {} }
	end
	for right_position, right_column_name in right.column_names do
		if right_column_name ~= right_on then -- Avoid duplicate join column
			local new_position = right_column_offset + right_position
			table.insert(joined_csv.column_names, right_column_name)
			joined_csv.column_map[right_column_name] = new_position
			joined_csv.columns[new_position] = { column_name = right_column_name, values = {} }
		end
	end

	local right_lookup: { [string]: { {string} } } = {}
	for _, right_row in right.rows do
		local key = right_row[right_index]
		if not right_lookup[key] then
			right_lookup[key] = {} :: any
		end
		table.insert(right_lookup[key], right_row)
	end

	for _, left_row in left.rows do
		local matching_right_rows: { { string } } = right_lookup[left_row[left_index]]
		if matching_right_rows then -- Only include rows with a match
			for _, right_row in matching_right_rows do
				local new_row: { string } = {}

				-- add left columns
				for column_position, left_value in left_row do
					table.insert(new_row, left_value)
					table.insert(joined_csv.columns[column_position].values, left_value)
				end

				-- add right columns (excludes the join column)
				for right_position, right_value in right_row do
					if right_position ~= right_index then
						local new_position = right_column_offset + right_position
						table.insert(new_row, right_value)
						table.insert(joined_csv.columns[new_position].values, right_value)
					end
				end

				table.insert(joined_csv.rows, new_row)
			end
		end
	end

	return joined_csv
end

--- converts a `Csv` back to a csv string
function csv.serialize(c: Csv): string
	local result = {}

	-- header row
	table.insert(result, table.concat(c.column_names, ","))

	-- format properly for saving to file
	for _, row in c.rows do
		for row_index, value in row do
			-- escape quotes w/ csv double quote ""text"" syntax
			if string.find(value, ",") or string.find(value, '"') then
				row[row_index] = '"' .. string.gsub(value, '"', '""') .. '"'
			end
		end
		table.insert(result, table.concat(row, ","))
	end

	return table.concat(result, "\n")
end

function csv.rows(c: Csv, first: number?, last: number?): { CsvRow }
	first = first or 1
	last = last or #c.rows

	local rows: { CsvRow } = {}
	-- sanity check bounds
	-- stylua: ignore
	local err_message =
		if first < 1 then 
			`Invalid row range: first ({first}) is below the minimum (1)`
		elseif last > #c.rows then 
			`Invalid row range: last ({last}) exceeds the total row count ({#c.rows})`
		elseif first > last then 
			`Invalid row range: first ({first}) is greater than last ({last}), expected first ≤ last`
		else 
			nil
	if err_message then
		error(err_message)
	end

	for row_index = first, last do
		table.insert(rows, row.new(c.column_names, c.rows[row_index]))
	end

	return rows
end

function csv.iterrows(c: Csv): () -> (number, CsvRow)
	local index = 0
	return function()
		index += 1
		if index > #c.rows then
			return nil :: any
		end
		return index, row.new(c.column_names, c.rows[index])
	end
end

function csv.itercolumns(c: Csv): () -> (number, CsvColumn)
	local index = 0
	return function()
		index += 1
		if index > #c.column_names then
			return nil :: any
		end

		local column_name = c.column_names[index]
		local column_values = {}

		for _, row_data in c.rows do
			table.insert(column_values, row_data[index] or "")
		end

		return index, column.new(column_name, column_values)
	end
end

csv.select = require("@self/select")

return csv
