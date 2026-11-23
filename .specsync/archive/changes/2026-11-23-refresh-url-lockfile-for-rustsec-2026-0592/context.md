---
change: refresh-url-lockfile-for-rustsec-2026-0592
artifact: context
---

# Context

GitHub Actions rejected PR #1144 in the `audit` job because the unchanged main lockfile contains `url` 2.5.8. RustSec advisory [RUSTSEC-2026-0592](https://rustsec.org/advisories/RUSTSEC-2026-0592.html) lists 2.5.9 as the first fixed release. The lockfile records `cookie_store` 0.22.0 as the immediate dependent.

Cargo 1.89's offline dry run resolves `url` 2.5.9 and `form_urlencoded` 1.2.3. Registry metadata gives `url` 2.5.9 a Rust 1.63 minimum and requires `form_urlencoded` ^1.2.3. The product path in scope is `Cargo.lock`.
