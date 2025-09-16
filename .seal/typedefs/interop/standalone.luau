--[=[
    Load code from standalone versions of *seal*.
]=]
export type standalone = {
    --[=[
        Extracts, loads, and executes luau bytecode from the standalone seal executable at `path`.

        This can be used to 'require' compiled standalone modules, except with no regular safety and no type safety...

        ## ⚠️ Safety

        This function is very unsafe because:
        1. You have to ensure that what the bytecode does is safe (it doesn't `rm -rf` your operating system or anything)
        2. You have to ensure the bytecode is valid (otherwise it will crash the luau vm).
    ]=]
    eval: (path: string, chunk_name: string) -> any,
    --- Returns `true` if `path` is a compiled standalone seal executable. Errors if the path doesn't exist, is permission denied, etc.
    check: (path: string) -> boolean,
    --- Extracts luau bytecode from the standalone seal executable at `path`. Errors if the path doesn't exist, is permission denied, etc.
    --- or the bytecode couldn't be extracted.
    extract: (path: string) -> buffer
}

return {} :: standalone