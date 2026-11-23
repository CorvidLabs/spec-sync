---
change: refresh-cookie-lockfile-for-rustsec-2026-0594
artifact: context
---

# Context

The RustSec dependency audit on main at `71f1867d` reports RUSTSEC-2026-0594 for the transitive `cookie 0.18.1`. The advisory lists `0.18.3` as a compatible patched version. An isolated preflight found that Cargo can select it with only a version and checksum change in the existing lockfile entry.

The product scope is `Cargo.lock` only. Preserve the existing manifest, resolution features, dependency edges, and audit warning policy. The patch refresh changes no SpecSync requirement or public API.
