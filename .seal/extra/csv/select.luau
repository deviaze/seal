local types = require("./types")
type Csv = types.Csv

type column_name = string

export type SelectQuery = {
	with_columns: { column_name },
	where: {
		[column_name]: (value: string) -> boolean,
	},
}

--[=[
	Selects rows and columns of `c` that match the `SelectQuery`, returning a new `Csv`.

	```luau
	export type SelectQuery = {
		with_columns: { column_name },
		where: {
			[column_name]: (value: string) -> boolean,
		},
	}
	```
]=]
local function csv_select(c: Csv, query: SelectQuery): Csv
	local new_rows = {}
	local new_columns = {}
	local new_column_map = {}

	for index, column_name in query.with_columns do
		new_column_map[column_name] = index
		new_columns[index] = {
			column_name = column_name,
			values = {},
		}
	end

	-- iterate through rows while applying filter fn
	for _, row_data in c.rows do
		local match = true

		for column_name, filter_fn in query.where do
			local column_index_in_row = c.column_map[column_name]
			local value = row_data[column_index_in_row]
			if not value then
				value = row_data[column_index_in_row - 1] -- hacky fix for weird last col issue?
			end
			if (value :: any) == nil then
				print(`missing value for column_name '{column_name}'?, tried index {column_index_in_row}`)
				print(row_data)
				error("csv missing requested column_name")
			end
			if not filter_fn(value) then
				match = false
				break
			end
		end

		-- if row matches, extract only requested columns
		if match then
			local selected_row = {}
			for index, column_name in query.with_columns do
				local value = row_data[c.column_map[column_name]]
				table.insert(selected_row, value)
				table.insert(new_columns[index].values, value)
			end
			table.insert(new_rows, selected_row)
		end
	end

	return {
		column_names = query.with_columns,
		column_map = new_column_map,
		columns = new_columns,
		rows = new_rows,
	}
end

return csv_select
