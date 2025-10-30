# Variables and (basic) Types

To define a variable (binding) in Luau, use the `local` keyword.

```luau
local cats = 1
```

The main primitive data types in Luau are `number`, `string`, `boolean`, `nil`,  `function`, `buffer`, `vector`, and `table`. Structs reflected from C or Rust are called `userdata` or `extern type`.

To prevent a binding from changing type later in the program, use an explicit type annotation:

```luau
local cats: number = 2
cats = "meow" -- TypeError: Type 'string' could not be converted into 'number'
```

If you don't see a TypeError here, ensure strict mode is enabled in your `.luaurc` or `.config.luau` file. Luau works best in strict mode. You can additionally enable strict mode on a file-by-file basis by adding `--!strict` to the top of your Luau file.

In practice, most `local` variables should not be explicitly annotated.

## Functions

To define a function, use the following syntax:

```luau
-- a local function:
local function increment(n: number, amount: number): number
    return n + amount
end
-- the exact same function, except using anonymous function syntax:
local increment = function(n: number, amount: number): number
    return n + amount
end
```

By convention, most function parameters and return types should be annotated.

To call a function, use parentheses:

```luau
local incremented = increment(1, 2) 
print(incremented) --> 3
```

Unlike many other languages, you can *omit* parentheses if you call a function with only a single argument -- if the argument is a string literal or a table literal:

```luau
local result: { string } = {}
local function push(s: string)
    table.insert(result, s)
end
push "hi"
push "bye"

local process = require("@std/process")
local result = process.run {
    program = "seal",
    args = { "run" },
}
```

Omitting parentheses is considered bad practice for string calls and usually discouraged for table calls except in the context of DSLs. *seal* considers table calls acceptable.

## Tables: structs, arrays, maps, and classes

The base data structure in Luau is the `table`. Tables can be used like structs, arrays, and maps (dictionaries), and can be composed to represent any data structure. All tables consist of key-value pairs and can contain any data type except `nil`.

An array-like table can be defined like this:

```luau
local values = {1, 2, 3, 4}
local strings = { "hi", "bye", "seals" }
```

An array-like table can contain values of different types:

```luau
local values: { number | string } = { "hi", 1, "bye", 2 }
```

A map-like table can be defined like this:

```luau
local cats: { [string]: string } = {
    first = "hi",
    last = "bye",
    seals = "seals",
    ["key with a space"] = "is allowed",
}

```

A struct-like table will always have the same keys (properties), and more properties cannot later be added to one without causing a TypeError.

```luau
local cat: {
    name: string,
    age: number
} = {
    name = "Taz",
    age = 12,
}
```

To use struct-like tables more effectively, use a `type` alias binding:

```luau
type Cat = {
    name: string,
    age: number,
}

local cat: Cat = {
    name = "Taz", -- Luau would throw a TypeError if any of these fields were incompatible
    age = 12,
}
```

Luau has special syntax for adding functions to tables:

```luau
local Cat = {}
function Cat.new(name: string, age: number)
    return {
        name = name,
        age = age,
    }
end
```
