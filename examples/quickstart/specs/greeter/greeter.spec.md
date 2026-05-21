---
module: greeter
version: 1
status: draft
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

- `pub fn greet(name: &str) -> String` — Returns a greeting for `name`.
  The greeting is a localized `"hello, <name>!"` string.

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

- 1.0 — initial spec for the canonical SpecSync quickstart example
