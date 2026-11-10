---
change: generate-and-scaffold-never-claim-files-another-spec-owns
artifact: testing
---

# Testing


Integration: spec `special` owns src/foo.rs; `scaffold foo` writes no spec (or a spec without
src/foo.rs); `new foo` likewise. Unit: candidate filtering against owned files.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-generator-009 | integration `scaffold_never_claims_a_file_another_spec_owns`, `scaffold_single_source_fallback_skips_an_owned_file`; existing `scaffold_auto_detects_single_source_file` still passes |
| REQ-cmd-generate-004 | generate modes bind from coverage (unchanged path); integration generate suite 44/44 green |
| REQ-cmd-scaffold-006 | integration `scaffold_never_claims_a_file_another_spec_owns` |
