---
id: CHG-0011-fix-windows-release-checksum-newline-portability
state: archived
type: bug_fix
base_commit: d6d8512f9a1d75f308df1e9a8f52b47ca9e839ee
---

# Fix Windows release checksum newline portability

## Intent

Fix Windows release checksum newline portability

## Affected Canonical Specs

- None

## Acceptance Criteria

- Windows release checksum files use LF and pass standard Unix shasum verification; the workflow verifies generated checksums before uploading artifacts; existing five-platform packaging remains unchanged.

## No-spec Rationale

The release workflow implementation changes, but no canonical Rust module API or documented SpecSync behavior changes.
