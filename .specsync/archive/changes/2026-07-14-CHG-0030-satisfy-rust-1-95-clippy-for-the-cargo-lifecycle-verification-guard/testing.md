---
change: CHG-0030-satisfy-rust-1-95-clippy-for-the-cargo-lifecycle-verification-guard
artifact: testing
---

# Testing

- Run the focused native Cargo command-classification regression test.
- Run the configured lifecycle verification command.
- Run `fledge trust verify` and require the Clippy lint step to pass.
