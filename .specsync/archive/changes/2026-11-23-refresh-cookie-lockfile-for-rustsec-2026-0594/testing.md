---
change: refresh-cookie-lockfile-for-rustsec-2026-0594
artifact: testing
---

# Testing

After scope approval, resolve `cookie 0.18.3` with the pinned toolchain. Inspect the lockfile diff and compare the resolved dependency graph and enabled features against main, accounting for the cookie version label. Run the repository’s RustSec audit with its existing policy and confirm RUSTSEC-2026-0594 clears.

Run the repository verification and pre-push lanes, the strict lifecycle audit, and the Trust and Attest checks on the resulting candidate before asking for implementation review.
