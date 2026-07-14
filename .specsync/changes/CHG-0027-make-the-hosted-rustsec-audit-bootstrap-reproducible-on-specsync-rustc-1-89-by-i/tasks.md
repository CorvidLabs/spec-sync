---
change: CHG-0027-make-the-hosted-rustsec-audit-bootstrap-reproducible-on-specsync-rustc-1-89-by-i
artifact: tasks
---

# Tasks

- [x] Capture the exact hosted installation failure and required Rust versions.
- [x] Configure the audit job with the repository's supported toolchain.
- [x] Pin cargo-audit 0.22.2 and enable its published lockfile.
- [x] Preserve the existing explicit advisory exception and fail all other findings.
- [x] Record successful workflow syntax validation and native dependency audit execution.
