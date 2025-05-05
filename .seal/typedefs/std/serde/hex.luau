export type Hex = {
	encode: (data: buffer | string) -> string,
	decode: (encoded: string) -> buffer,
}

return {} :: Hex