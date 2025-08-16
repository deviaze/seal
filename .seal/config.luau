local config = {
    -- entry_path = "./src/main.luau",
    test_path = "./tests/run.luau",
}

export type SealConfig = {
    --- Script that `seal run` runs; usually the entrypoint to your codebase.
    --- Defaults to `./src/main.luau`.
    entry_path: string?,
    --- Script that `seal test` runs; usually a test runner.
    test_path: string?,
}

return config :: SealConfig