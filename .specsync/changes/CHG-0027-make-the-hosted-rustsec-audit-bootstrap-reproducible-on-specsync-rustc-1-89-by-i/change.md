---
id: CHG-0027-make-the-hosted-rustsec-audit-bootstrap-reproducible-on-specsync-rustc-1-89-by-i
state: verifying
type: operations
base_commit: c98d29810f78abcdd6a2fec9b137667d3ab2fc5b
---

# Make the hosted RustSec audit bootstrap reproducible on SpecSync rustc 1.89 by installing cargo-audit 0.22.2 with its published lockfile and retaining the acknowledged RUSTSEC-2024-0384 exception.

## Intent

Make the hosted RustSec audit bootstrap reproducible on SpecSync rustc 1.89 by installing cargo-audit 0.22.2 with its published lockfile and retaining the acknowledged RUSTSEC-2024-0384 exception.

## Affected Canonical Specs

- None

## Acceptance Criteria

- Hosted CI installs cargo-audit 0.22.2 successfully on rustc 1.89 using the crate lockfile; cargo audit runs against Cargo.lock; the existing RUSTSEC-2024-0384 exception is explicit and no additional advisory is accepted; the audit job passes.

## No-spec Rationale

This changes only CI tool installation and invocation; it does not alter SpecSync runtime behavior or public contracts.
