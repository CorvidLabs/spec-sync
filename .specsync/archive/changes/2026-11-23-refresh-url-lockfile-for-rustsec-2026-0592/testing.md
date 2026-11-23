---
change: refresh-url-lockfile-for-rustsec-2026-0592
artifact: testing
---

# Testing

- Run `cargo update -p url --precise 2.5.9` using the pinned Rust toolchain. Inspect the product diff and verify that its lockfile entries change for `url` and `form_urlencoded`.
- Run `cargo audit` against the current RustSec database and confirm that RUSTSEC-2026-0592 clears.
- Run `fledge lanes run verify`, SpecSync strict validation with full coverage, `specsync change audit --strict`, `fledge lanes run pre-push`, and `fledge trust verify` against the candidate before the scoped human review.
