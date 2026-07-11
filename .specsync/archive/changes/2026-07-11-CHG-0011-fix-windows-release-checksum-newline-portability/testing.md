---
change: CHG-0011-fix-windows-release-checksum-newline-portability
artifact: testing
---

# Testing

- Parse the workflow as YAML.
- Exercise the portable checksum verifier locally against valid LF and invalid CRLF fixtures.
- Run the repository CI matrix, including the Windows runner and workflow validation.
- Confirm strict SpecSync lifecycle and canonical spec checks remain green.

## Local Evidence

- `actionlint .github/workflows/release.yml` passes.
- PyYAML parses the workflow successfully.
- The exact embedded verifier accepts a valid LF checksum record.
- The exact embedded verifier rejects the reproduced CRLF record with exit 1.
- `fledge run fmt` passes.
- `fledge run test` passes all 1,527 unit and 187 integration tests.
