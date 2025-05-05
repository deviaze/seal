export type require_debug = {
	load: (chunk_name: string, to_path: string) -> unknown,
	get_luaurc: (chunk_name: string) -> unknown,
}

return {} :: require_debug