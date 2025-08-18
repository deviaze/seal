# Non-goals/out of scope

- Fully featured standard library for all usecases: `seal` is primarily suited for high level scripting and light general purpose programming.
- We don't want to add every single hash algorithm, nor bind to every single part of Rust's standard library—providing too many options might end up confusing to the average user. Use [Zune](<https://github.com/Scythe-Technology/zune>) instead if you need lower level bindings.
- Async webservers. *seal* is not an async runtime and thus cannot scale up enough for webservers, but if you want to write one (or a Discord bot) in Luau and need async, I highly recommend using Zune for that instead.
- Premature optimization. Although *seal* is pretty-to-very fast, it might not be the absolute fastest way to use Luau due to `mluau`'s slight safety overhead.
  - Still way faster than Python.
  - On the other hand, because `seal` doesn't have any tokio or async overhead, its standard library should be faster than `Lune`'s, and because of its parallelism model, true multithreaded programs in `seal` should be more stable than programs that rely on Lune's `task` library and IO.
