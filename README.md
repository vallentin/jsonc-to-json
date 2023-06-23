# jsonc-to-json

[![CI](https://github.com/vallentin/jsonc-to-json/workflows/CI/badge.svg)](https://github.com/vallentin/jsonc-to-json/actions?query=workflow%3ACI)
[![Latest Version](https://img.shields.io/crates/v/jsonc-to-json.svg)](https://crates.io/crates/jsonc-to-json)
[![Docs](https://docs.rs/jsonc-to-json/badge.svg)](https://docs.rs/jsonc-to-json)
[![License](https://img.shields.io/github/license/vallentin/jsonc-to-json.svg)](https://github.com/vallentin/jsonc-to-json)

<!-- cargo-rdme start -->

Simple library for converting [JSON with Comments] into [JSON],
in short it removes the following:

- Line comments, e.g. `// Line Comment`
- Block comments, e.g. `/* Block Comment */`
- Trailing commas, e.g. `[1,2,3,,]` -> `[1,2,3]`

**Note:** The implementation does not use a full-blown [JSON with Comments]
parser. Instead it uses a [JSON with Comments] tokenizer, which makes
conversion a lot faster.

Currently `#![no_std]` is not supported. It will however be added, when
some upstream changes have been applied.

See [`jsonc_to_json()`] for more information.

## Example

```rust
use jsonc_to_json::{jsonc_to_json, jsonc_to_json_into};

let jsonc = "{\"arr\": [1, 2,/* Comment */ 3, 4,,]}// Line Comment";

let json = jsonc_to_json(jsonc);
println!("{}", json);

// Alternatively, use `jsonc_to_json_into()` to reuse an
// already allocated `String`
let mut json = String::new();
jsonc_to_json_into(jsonc, &mut json);
println!("{}", json);
```

Both outputs the following:

```text
{"arr": [1, 2, 3, 4]}
```

## Non-Allocating Iterator Example

Non-allocating [`Iterator`] that yields string slices of
valid [JSON].

```rust
use jsonc_to_json::jsonc_to_json_iter;

let jsonc = r#"{foo}/**/[1,2,3,,]"bar""#;

let mut iter = jsonc_to_json_iter(jsonc);
assert_eq!(iter.next(), Some("{foo}")); // Line comment was removed
assert_eq!(iter.next(), Some("[1,2,3")); // Trailing commas was removed
assert_eq!(iter.next(), Some("]\"bar\""));
assert_eq!(iter.next(), None);
```

[JSON with Comments]: https://code.visualstudio.com/docs/languages/json#_json-with-comments
[JSON]: https://www.json.org/json-en.html

<!-- cargo-rdme end -->

[`jsonc_to_json()`]: https://docs.rs/jsonc-to-json/*/jsonc-to-json/fn.jsonc_to_json.html
[`Iterator`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html
