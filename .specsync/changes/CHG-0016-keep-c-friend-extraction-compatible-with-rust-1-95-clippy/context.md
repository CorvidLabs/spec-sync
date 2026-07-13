---
change: CHG-0016-keep-c-friend-extraction-compatible-with-rust-1-95-clippy
artifact: context
---

# Context

Rust 1.95 added a `collapsible_match` Clippy finding for the C++ extractor's
public-friend branch. The existing nested condition and a match guard select
the same nodes. This is a compiler-lint compatibility adjustment, not a parser
contract or extraction behavior change.
