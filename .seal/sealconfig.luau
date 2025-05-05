local seal_config: SealConfig = {
	entrypoint = "./src/main.luau",
	test_runner = "./tests/run_tests.luau",
	package_manager = "None",
}

export type SealConfig = {
	--- decides which file `seal run` runs
	entrypoint: string,
	--- doesn't do anything yet
	package_manager: string | "rokit" | "pesde" | "None",
	--- decides what `seal test` runs
	test_runner: string?,
}

return seal_config