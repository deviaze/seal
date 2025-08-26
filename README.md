<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD026 -->

# *seal* <small>the cutest runtime for luau</small>

## is a highly reliable cross-platform scripting and automation runtime.

Use *seal* to write shell-like, easily-deployable programs with [Luau](https://luau.org), a simple, dependable, and extremely fast scripting language with strict typechecking and good tooling support.

## Features

- **Batteries-included setup** that helps you get straight into working on your script, shim, or project.
- Expressive **standard library** with easy-to-use **fs**, **process/shell**, **http**, **multithreading**, etc. APIs.
- First-class editor support for autocomplete, inline documentation, and type safety.

... and more; see the [full list of features](docs/standard-library/index.md).

## Upcoming features (0.1.0 -> 0.2.0 roadmap)

- 0.0.7: Standalone project compilation for portability (with Darklua).
- 0.1.0: Integrated webview bindings/UI library for GUI scripts.
- Automation:
  - Keyboard rebinding bindings to write custom keyboard mapping layers.
  - Mouse/keyboard automation if possible.

## Install

To use *seal*, you need 3 things:

1. A text editor that supports the Luau Language Server, such as `VSCode`, `Zed`, or `nvim`.
2. [Luau Language Server](https://github.com/JohnnyMorganz/luau-lsp), which provides amazing inline documentation, static analysis and typechecking support, and diagnostics for the Luau programming language. *seal* automatically sets up everything for Luau Language Server when you run `seal setup/project/script/custom`.
3. The *seal* executable, which you can find packaged for your platform here: <https://github.com/deviaze/seal/releases/latest>.

## Setup

1. Make *seal* available in your `PATH` so you can use it in a terminal.

<details>
<summary>How to add seal to PATH?</summary>

Option 1 - using *seal*

1. Save this *seal* script to your Downloads folder: [seal_install.luau](examples/seal_install.luau)
2. Read it so you know how it works! Or modify the path so it moves seal where you want it to.
3. Open your Downloads folder in your terminal and run `./seal ./seal_install.luau`
4. On Windows, add the `~\.local\bin` path to your `$PROFILE` file with the instructions provided.
5. Open a new terminal and make sure `seal --help` works.

Option 2 - Windows Terminal on Windows:

1. Open Windows Terminal (PowerShell)
2. Move `seal` somewhere permanent like `C:\Users\<USERNAME>\.local\bin`:
   1. Open your Downloads folder (`cd "~\Downloads"` or `cd "~\OneDrive\Downloads"`) and run `mv .\seal.exe "~\.local\bin\seal.exe"`
3. Run `code $PROFILE` to open your powershell profile in vscode.
4. Add `$env:Path += ";C:\Users\<USERNAME>\.local\bin"` near the bottom or wherever you add your paths.
5. Close and reopen your Windows Terminal and run `seal --help` to make sure seal is available.

</details>

2. Make a new project directory with `md` or `mkdir`.
3. `cd` into that directory and run `seal setup` to create a new project.
4. Open the project in your code editor (`code .` for vscode).

## Usage

*seal* codebases can be either *Projects*, *Scripts*, or single files.

To generate a codebase in your current directory, use `seal setup <codebase>`, where codebase can be `project`, `script`, or `custom`.

- To start a project in your current directory, run `seal setup project`.
- To run a project, run `seal run` or `seal r`.
- To run a single file with seal, run `seal ./<filename>.luau`.

Check out the full [usage instructions](docs/usage.md) for more.

## Programming and Standard Library

Although seal provides some builtin globals (such as `p`, `pp`, `channel` (in a child thread), and `script`), most features are in the standard library. You can import stdlibs like so:

```luau
local fs = require("@std/fs")
local colors = require("@std/colors") -- (the most important one tbh)

-- some libs are nested:
local input = require("@std/io/input")
```

[See the docs](docs/libraries_and_programming.md) for all available libraries and APIs.

## Goals

- Focus on **high-level scripting**, shell and input automation, and being the best shell/Python alternative for Luau.
- Be **extremely helpful and user friendly**. When you run into trouble, *seal* should tell you *exactly* what went wrong with a custom, handcrafted recommendation, warning or error message.
- **Reliability and transparency.** *seal* should *\*just work\** and never cause undocumented blocks, panics, nor unexpected behavior. *seal*'s internals should be readily accessible so it remains easy to understand, hackable, customizable, and fixable by its users.
<!-- - *seal* should integrate well with tooling, other languages, and other runtimes. Setting up new projects should be instantaneous, and adding *seal* to existing (*seal* and non-*seal*) projects should be just as easy. -->

## Contributing

I would greatly appreciate any contributions and feedback, including issues, PRs, or even messages on Discord saying "hey can you add this to seal"!

See the [contribution instructions](CONTRIBUTING.md) if you'd like to contribute to seal's codebase :3

If you find a bug in seal, please make a bug report issue on GitHub and ping me on Discord.

~~*seal* makes writing scripts and programs fun and easy, and runs them seally fast.~~
