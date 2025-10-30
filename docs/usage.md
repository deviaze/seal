# Usage

To run a codebase at its entry path, use `seal run`.

To run a single file, use `seal ./<filename>.luau`

To evaluate code from a string src, use `seal eval '<src>'`

To create a new codebase at the current directory, use one of:

- `seal setup project` short: `seal sp`
- `seal setup script` short: `seal ss`
- `seal setup custom` short: `seal sc`) (interactive)

For help, use `seal --help` or `seal help <command>`

## Codebases

*seal* codebases can be either *Projects*, *Scripts*, or single files.

The general setup for a codebase should follow:

1. Open a terminal
2. `mkdir/md ProjectName`
3. `cd ProjectName`
4. `seal sp` or `seal ss` (setup a project or script codebase)
5. `code .` or `zeditor .`

### Projects

Use a **Project** codebase when you want to use *seal* as the primary runtime for your project.

This option will generate a `.seal` directory containing seal's configuration and all typedefs locally for easy portability and standalone compilation, a `src` dir, a `.luaurc` Luau configuration file, a `.vscode/settings.json`, and will start a new `git` repository if one doesn't already exist.

### Scripts

Use a **Script** codebase when you want to add *seal* to an existing project to run build, glue, or extension scripts.

This option generates a `.seal` directory locally for seal configuration, but will otherwise link to user-wide typedefs in `~/.seal/typedefs/*`.

Additionally, the project's `.vscode/settings.json` and `.luaurc` will also be created or updated to include *seal*'s typedefs and default config.

#### Configuring codebases

Both Project and Script codebases should have a `.seal/config.luau` file, which you can modify to set a codebase entry path, test runner path, etc.

The default config is:

```luau
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

```

### Running single files

To run a `.luau` file with seal, use `seal <filename_with_ext>` (like `seal ./get_the_endpoint.luau`).

### Evaluating code from the command line

To evaluate code with seal, use `seal eval '<string src>'`. `seal eval` comes with the `fs`, `http`, and `process` libs loaded in for convenience. An interactive REPL is planned for the future.
