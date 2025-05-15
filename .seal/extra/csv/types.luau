-- need a separate file to non-recursively share types

type column_name = string
export type Csv = {
	column_names: { string },
	column_map: {
		[column_name]: number,
	},
	columns: {
		[number]: {
			column_name: string,
			values: { string },
		},
	},
	rows: {
		[number]: { string },
	},
}

return nil