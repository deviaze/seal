local seal_config: SealConfig = {
	entry_path = "./src/main.luau",
	test_path = "./tests/run.luau",
	package_manager = "None",
}

export type SealConfig = {
	--- entrypoint to the project; decides which file `seal run` runs
	entry_path: string,
	--- doesn't do anything yet
	package_manager: string | "rokit" | "pesde" | "None",
	--- decides what `seal test` runs
	test_path: string?,
}

return seal_config