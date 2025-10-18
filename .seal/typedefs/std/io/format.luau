--[=[
    Format objects for pretty printing to stdout/stderr.
]=]
export type format = setmetatable<{
    --[=[
        Formats `item` in the same way as `print` or `pp`.
    ]=]
    pretty: (item: unknown) -> string,
    --[=[
        Like pretty printing but without colors.
    ]=]
    simple: (item: unknown) -> string,
    --[=[
        Prints the debug representation of `item`, equivalent to using `{:?}` in Rust.
    ]=]
    debug: (item: unknown) -> string,
    --[=[
        Removes ANSI color codes from a pretty formatted string.
    ]=]
    uncolor: (s: string) -> string,
}, { 
    __call: (self: any, item: unknown) -> string,
}>

return {} :: format