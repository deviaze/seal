
# Contributing

I would greatly appreciate any contributions and feedback, including issues, PRs, or even messages on Discord saying "hey can you add this to seal"!

## Adding new libraries

### seal extras (@extra)

A library is a good candidate for @extra if:

- It's mostly implemented or implementable in Luau with or without *seal*.
- Users may want to customize it (@std libs can't be customized at runtime but @extra can)
- Users may want to run it directly as a CLI program.

To add a library to `@extra`, just add it, make sure it works, and open a pull request.

### Standard Library (@std)

A library is a good candidate for `@std` if:

- There's a good chance *everyone* would want/need it
- It's mostly implemented in Rust, and would be difficult or impossible to translate to other Luau runtimes
- It'd be impossible to implement in user code.
- There's a Rust crate for it that we could add easily and isn't async and won't massively increase seal's executable binary size.
- We'll need to rely on it internally in *seal* (this is why `@std/semver` is in the standard library and `@extra/tt` isn't)

#### How to contribute a standard library

1. Implement the code in Rust or Luau in `./src/std_*libname*`. We use `std_*` prefixes to avoid name clashes with the Rust standard library.
2. Use the newest/most recently refactored existing libraries (especially `@std/process`, `@std/fs`, and `@std/thread`) as templates for your Rust code structure.
3. Rust error handling:
   1. Don't use `.unwrap` or `.expect` in Rust code unless you're 99-100% sure it won't panic.
   2. Every error case reachable by user code should be covered by a `wrap_err!`, with an ideally handcrafted error message and the luau-side function_name in front.
   3. If there's an invariant that if reached means there's actually a bug, use an explicit `panic!` or `unreachable!`.
   4. Users should not see the word "Lua" in an error message (this comes from directly bubbling up `mluau` errors).
4. Casing conventions bikeshedding
   1. Top level library functions should be luaucase and short. If they end up ugly/unreadable then either rename them, put them in a sublib, or make them snake_case. New APIs should match the existing APIs, for example functions that purposely try not to wrap_err! should be named `libname.try_*` in snake_case.
   2. Try to keep the names short and sweet so we don't have to combine words in luaucase: like `json.raw` instead of `json.encoderaw`.
   3. Library properties should be snake_case unless you can keep them to one word.
   4. Object-like method names should be snake_case unless they match or partially match a luaucase api.
5. please don't run a formatter over the whole codebase.
   1. I don't mind if `wrap_err!`s go off the RHS of the page if that means vertical space is better used for code.
   2. On the other hand, let's try to keep non-wrap_err! code and comments to 85-110 colwidth?
6. Documentation goes in `./.seal/typedefs/std/*`.
   1. For documentation, try to stick with the newer docs headers like seen in the `@std/process` API docs.
   2. No Moonwave `@attributes`, they make code less readable and we don't use Moonwave.
7. Register the library in `./src/require/mod.rs` and `./.seal/typedefs/init.luau` in 3 places:
   1. The Big Beautiful Table (BBT) in `./src/require/mod.rs`, which handles when the library is required directly (`@std/mylib`)
   2. the Small Beautiful Table in `./src/require/mod.rs`, which handles when the entire `@std` is required at once.
   3. in `./.seal/typedefs/init.luau` to provide `require` support when the entire `@std` is required at once.
8. Add tests in `./tests/luau/std/libname/*` or `./tests/luau/std/libname.luau`.
