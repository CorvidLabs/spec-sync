---
change: CHG-0016-keep-c-friend-extraction-compatible-with-rust-1-95-clippy
artifact: testing
---

# Testing

- `fledge run fmt` passes.
- `fledge run lint` passes on Rust 1.95 with `-D warnings`.
- `cargo test exports::cpp::` passes all 12 focused extractor tests.
