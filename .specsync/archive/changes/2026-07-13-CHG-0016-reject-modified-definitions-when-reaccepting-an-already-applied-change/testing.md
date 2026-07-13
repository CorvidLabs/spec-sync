---
change: CHG-0016-reject-modified-definitions-when-reaccepting-an-already-applied-change
artifact: testing
---

# Testing

`REQ-change-017` is covered by `change::tests::reaccept_rejects_definition_changes_after_canonical_application` and the expanded `change::stale_accepted_change_reopens_through_cli_with_deterministic_audit_json` integration sequence.

Both tests create real accepted evidence, mutate governed delivery input, reopen with explicit audit fields, change the selected definition, record a fresh definition approval and verification, and prove reacceptance rejects the changed contract. The CLI test then restores the original definition and proves delivery-only reacceptance still succeeds without double application.

Repository validation runs Rustfmt, Clippy with denied warnings, all unit and integration tests, release build, RustSec audit, strict SpecSync at 100% coverage, documentation tests/lint/build, VS Code compile/package, and full Trust.
