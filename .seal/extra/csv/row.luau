local str = require("@std/str")

type CsvRowStruct = { { [string]: string } }
type CsvRowImpl = {
	values: (self: CsvRow) -> { string },
	display: (self: CsvRow) -> string,
}

export type CsvRow = setmetatable<CsvRowStruct, {
	__index: CsvRowImpl,
}>

local row = {}

local impl = {}

function impl.values(self: CsvRow): { string }
	local result: { string } = {}
	for column_index, map in self do
		for _col, value in map do
			table.insert(result, value)
		end
	end
	return result
end

function impl.display(self: CsvRow): string
	local col_widths = {}

	-- determine column widths
	for column_index, map in self do
		for column_name, value in map do
			local max_width = math.max(str.width(column_name), str.width(value))
			col_widths[column_index] = math.max(col_widths[column_index] or 0, max_width)
		end
	end

	-- pad/format row
	local function format_row(row_data: { string })
		local formatted = {}
		for column_index, value in row_data do
			table.insert(
				formatted,
				value .. string.rep(" ", math.max(0, col_widths[column_index] - str.width(value)))
			)
		end
		return table.concat(formatted, "  ")
	end

	-- separator line that matches padding
	local separator_parts = {}
	for _, width in col_widths do
		table.insert(separator_parts, string.rep("-", width))
	end
	local separator = table.concat(separator_parts, "  ")

	local headers, values = {}, {}
	for column_index, map in self do
		for column_name, value in map do
			headers[column_index] = column_name
			values[column_index] = value
		end
	end

	return format_row(headers) .. "\n" .. separator .. "\n" .. format_row(values)
end

function row.new(column_names: { string }, row_data: { string }): CsvRow
	local row_list = {}

	-- map like { [column_name]: value }
	for column_index, column_name in column_names do
		row_list[column_index] = { [column_name] = row_data[column_index] or "" }
	end

	return setmetatable(row_list, { __index = impl })
end

return row
