# Usage

To run a codebase at its entry path, use `seal run`.

To run a single file, use `seal ./<filename>.luau`

To evaluate code from a string src, use `seal eval '<src>'`

To create a new codebase at the current directory, use one of:

- `seal setup project` (`seal sp`)
- `seal setup script` (`seal ss`)
- `seal setup custom` (`seal sc`) (interactive)

For help, use `seal --help` or `seal help <command>`

## Codebases

*seal* codebases can be either *Projects*, *Scripts*, or single files.

The general setup for a codease should follow:

1. Open a terminal
2. `mkdir/md ProjectName`
3. `cd ProjectName`
4. `seal sp` or `seal ss` (setup a project or script codebase)
5. `code .` or `zeditor .`

### Projects

Use a **Project** codebase when you want to use *seal* as the primary runtime for your project; this option will generate `.seal` directory, all typedefs locally for easy portability (and soon, compilation), a `src` dir, a `.luaurc`, a `.vscode/settings.json`, and will start a new `git` repository if one doesn't already exist.

Run `seal setup project` or `seal sp` to generate a **Project** codebase at your current directory.

### Scripts

Use a **Script** codebase when you want to add *seal* to an existing project to run build or glue scripts, without making *seal* the whole point of your project. This option generates a `.seal` directory locally for seal configuration, but will otherwise link to user-wide typedefs in `~/.seal/typedefs/*`. `.vscode/settings.json` and `.luaurc`s will also be created or updated to include *seal*'s typedefs and default config.

Run `seal setup script` or `seal ss` to add a **Script** codebase to your current directory.

#### Configuring codebases

Both Project and Script codebases should have a `.seal/config.luau` file, which you can modify to set a codebase entry path, test runner path, etc.

To run a codebase at its entry path, use `seal run` or `seal r`. Note this command is similar to `cargo run` in Rust, and isn't used to run single files.

Automatic setup for Zed is not fully ready yet, but all the other settings are available for config when you run `seal setup custom`.

### Running single files

To run a `.luau` file with seal, use `seal <filename_with_ext>` (like `seal ./get_the_endpoint.luau`).

### Evaluating code from the command line

To evaluate code with seal, use `seal eval '<string src>'`. `seal eval` comes with the `fs`, `http`, and `process` libs loaded in for convenience. An interactive REPL is planned for the future.
