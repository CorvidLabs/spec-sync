---
change: CHG-0027-make-the-hosted-rustsec-audit-bootstrap-reproducible-on-specsync-rustc-1-89-by-i
artifact: design
---

# Design

The job installs the repository toolchain explicitly, installs the known-compatible `cargo-audit 0.22.2` release with `--locked`, and invokes the audit command directly. The crate's published lockfile makes its transitive installation graph reproducible. The direct command keeps the existing single advisory exception visible in version-controlled workflow policy and fails for every other vulnerability.

No product source, dependency lockfile, release artifact, or public interface changes.
