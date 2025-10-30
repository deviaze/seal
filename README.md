# seal - a cute cross-platform scripting runtime

Use *seal* to write fun, maintainable, and easily-deployable programs in [Luau](https://luau.org), a simple, dependable, and extremely fast scripting language with good typechecking and tooling support.

Check out the [standard library reference](docs/standard-library/index.md) for all current features and APIs.

## Goals

- Provide a **simple but expressive API** that allows you to get straight into working on your script, shim, or project.
- Be **unapologetically helpful and user friendly**. When you run into trouble, *seal* should tell you *exactly* what went wrong with a custom, handcrafted recommendation, warning, or error message.
- **Reliability and transparency.** *seal* should *\*just work\** and never cause undocumented blocks, panics, nor unexpected behavior. *seal*'s internals should be readily accessible so it remains easy to understand, hackable, customizable, and fixable by its users.

## Install

See the [install instructions](docs/install.md) for a detailed walkthrough of getting *seal* on your system. Basically you just need a text editor (VSCode, Zed, or nvim), [Luau Language Server](https://github.com/JohnnyMorganz/luau-lsp) installed in your text editor, and the [latest release](<https://github.com/deviaze/seal/releases/latest>) of *seal* in your `PATH`.

## Usage

To start a new project with *seal*, make a new directory, run `seal setup project` inside it, and open it up with `code .`. The codebase's entry point should be at `./src/main.luau`. To use Zed instead of VSCode, run `seal setup custom` instead and follow the prompts.

*seal* codebases can be either projects or scripts; use a *project* codebase if the entire project will be written with *seal* and/or will be compiled to a standalone program, and use a *script* codebase if you want to use *seal* scripts in an existing or primarily not-*seal* project.

- `seal ./filename.luau` runs a Luau file with *seal*.
- `seal run` runs the project at your current working directory.

Check out the full [usage instructions](docs/usage.md) for more.