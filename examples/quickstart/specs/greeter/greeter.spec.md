---
module: greeter
version: 2
status: active
files:
  - src/lib.rs
---

# greeter

## Purpose

A trivial greeter module. Exists as the canonical SpecSync 5-minute
example — small enough to read in 30 seconds, real enough to
demonstrate that SpecSync validates code against the spec in both
directions.

## Public API

| Name | Kind | Description |
|---|---|---|
| `greet` | function | Returns the greeting `"hello, <name>!"` for `name`. |

## Invariants

- `greet` is **pure**: same input always produces the same output, no
  IO or hidden state.
- The returned string always contains the input `name` verbatim.

## Behavioral Examples

```rust
assert_eq!(greet("world"), "hello, world!");
assert_eq!(greet(""), "hello, !");
assert_eq!(greet("Leif"), "hello, Leif!");
```

## Error Cases

`greet` cannot fail. No panics, no errors. The function accepts any
`&str` including the empty string.

## Dependencies

- `std::format!` — string formatting (Rust standard library)

## Change Log

- 2.0 — `status: active` and `files:` relative to the example root, so the
  example validates standalone and export drift is reported
- 1.0 — initial spec for the canonical SpecSync quickstart example
