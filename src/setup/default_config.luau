local config = {
    entry_path = "./src/main.luau",
    test_path = "./tests/run.luau",
    seal_version = "<SEAL_VERSION_REPLACE>"
}

export type SealConfig = {
    --- Script that `seal run` runs; usually the entrypoint to your codebase.
    --- Defaults to `./src/main.luau`.
    entry_path: string?,
    --- Script that `seal test` runs; usually a test runner.
    test_path: string?,
    --- semver version of seal this project/typedefs expects to run on
    seal_version: string,
}

return config :: SealConfig