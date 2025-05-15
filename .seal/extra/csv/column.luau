local column = {}

local str = require("@std/str")
local row = require("./row.luau")

type ColumnStruct = {
    column_name: string,
    values: { string },
}

type ColumnImpl = {
    row: (self: CsvColumn, index: number) -> row.CsvRow,
    display: (self: CsvColumn) -> string,
}

export type CsvColumn = setmetatable<ColumnStruct, {
    __index: ColumnImpl
}>

local impl = {}

--- get the `CsvRow` of a column value at `index`
function impl.row(self: CsvColumn, index: number): row.CsvRow
    if index < 1 or index > #self.values then
        error(`Invalid row index: {index}`)
    end

    -- Pass column name as a list and indexed value as row data
    return row.new({ self.column_name }, { self.values[index] })
end

function impl.display(self: CsvColumn): string
    local result = {}
	local col_width = str.width(self.column_name) do
		for _, value in self.values do
			col_width = math.max(col_width, str.width(value))
		end
	end

    table.insert(result, self.column_name .. string.rep(" ", col_width - str.width(self.column_name)))
    table.insert(result, string.rep("-", col_width))
    for _, value in self.values do
        table.insert(result, value .. string.rep(" ", col_width - str.width(value)))
    end

    return table.concat(result, "\n")
end

function column.new(column_name: string, values: { string }): CsvColumn
    return setmetatable({
        column_name = column_name,
        values = values,
    }, { __index = impl })
end

return column
