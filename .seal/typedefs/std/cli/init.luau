--[=[
    Power up your CLI (command line interface) applications with argument parsing and input prompting :)

    You can import this library's sublibs directly via `@std/cli/args` and `@std/cli/prompt`

    - To easily handle command line arguments, flags, and --help, use `cli.args.parse`
    - To ask users personal questions about their lives, use `cli.prompt.*`.

]=]
export type cli = {
    prompt: typeof(require("@self/prompt"))
}