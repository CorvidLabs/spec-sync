---
change: scaffold-prints-the-empty-files-warning-when-every-candidate-source-is-owned-by-another-spec
artifact: testing
---

# Testing

Integration tests in `tests/integration/config.rs` run the built binary against a
fixture where another spec already lists the only candidate source file.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-cmd-scaffold-006 | integration `scaffold_never_claims_a_file_another_spec_owns`, `scaffold_single_source_fallback_skips_an_owned_file` (stderr warning and `files: []`) |
| REQ-generator-009 | integration `new_never_claims_an_owned_flat_file` (stderr warning and `files: []`) |
