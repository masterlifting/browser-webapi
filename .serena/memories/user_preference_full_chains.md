## User Rust Style Preference: Functional Chain With Inner Steps

- Prefer the `close(...)`-style pattern in `src/browser/tab/api.rs`: define several small inner helper functions (sync `fn` and async `async fn`) and then compose them at the end of the public function with a single combinator chain (typically `future::ready(...)` + `.and_then(...)`, plus `.map_err`/`.map_ok`/`.map`).
- Carry state forward via tuples returned by helpers.
- Avoid imperative control flow when a chain reads clearly.
