export type EvalOptions = {
    name: string?,
    stdlib: ("seal" | "safe" | "none")?,
}

export type luau = {
    --> luau.eval(src: string, options: EvalOptions?)
    --[=[
        Evaluate Luau source code from a string in the current Luau VM.

        By default, this function evaluates in `"safe"` mode with only Luau's standard library (minus some deprecated environment breaking functions).

        ### `EvalOptions` options:

        `name` represents the `chunk_name` of the evaluated src.

        `stdlib` can be one of the following (or left unspecified, in which it defaults to `"safe"`):

        - `"safe"` - The evaled code will have access to most libraries/functions that come with Luau, 
            but nothing that can access your file system or the internet.
        - `"seal"` - The evaled code will have access to anything seal can do, from accessing environment variables to creating an infinite number of empty files in your home directory.
        - `"none"` - Disable every single global Luau comes with, including `tostring` and `print`.

        ## Returns

        Either whatever the source code evaluates to (`unknown`), or a tostringable userdata instance representing
        an error that occurred when evaluating the code, such as a syntax error or runtime error.

        ## Errors
        - if the code cannot be evaluated, but not if it contains a syntax error or errors at runtime.

        ## Usage

        ```luau
        local luau = require("@std/luau")
        local src = [[return { meow = 2 }]]
        local res = luau.eval(src)
        local data: { meow: number } = {}
        if typeof(res) == "error" then
            print(`error running code: {res}`)
        else
            data.meow = (res :: any).meow
        end
        ```
    ]=]
    eval: (src: string, options: EvalOptions?) -> unknown | error,
    --> luau.eval_unsafe(src: string | buffer, options: EvalOptions?)
    --[=[
        Same as `luau.eval`, except can also accept bytecode as a string or buffer.

        ## ⚠️ Safety

        This function is unsafe. You are responsible for **passing valid Luau bytecode**, and therefore
        you should trust or check the bytecode you pass to this function.

        If you pass invalid bytecode as `src`, seal will 💥 ***crash*** 💥 from an ***illegal hardware instruction***
        and *coredump*.
    ]=]
    eval_unsafe: (src: string | buffer, options: EvalOptions?) -> unknown | error,
    --> luau.bytecode(src: string)
    --[=[
        Compiles `src` to Luau bytecode.
    ]=]
    bytecode: (src: string) -> buffer | error,
    --> luau.require_resolver()
    --[=[
        Returns *seal*'s require resolver implementation used internally.
    ]=]
    require_resolver: () -> {
        --- Resolve a Luau require alias (`requested_path`) relative to `requiring_file_path` to find its location on the filesystem.
        resolve: (requested_path: string, requiring_file_path: string) -> { err: string, path: nil } | { path: string, err: nil },
        get_aliases: (requiring_file_path: string) -> ({ LuaurcAliases }?, string?),
        expand_aliases: (requested_path: string, aliases_by_luaurc: { LuaurcAliases }) -> (string?, string?),
    }
}

export type LuaurcAliases = {
	path: string,
	aliases: {
		[string]: string
	}
}

return {} :: luau