---
change: CHG-0027-make-the-hosted-rustsec-audit-bootstrap-reproducible-on-specsync-rustc-1-89-by-i
artifact: plan
---

# Plan

1. Replace the action's unlocked installation with an explicit rustc 1.89 and locked cargo-audit 0.22.2 install.
2. Invoke `cargo audit` with only the repository's existing RUSTSEC-2024-0384 exception.
3. Validate the workflow syntax, native audit policy, strict SpecSync lifecycle, and hosted audit job.
