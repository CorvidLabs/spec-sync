---
change: split-parser-frontmatter-issue-validation-symbols-and-sections-into-focused-private-modules
artifact: testing
---

# Testing

Retain all 69 original parser tests and compare the compiled leaf-name inventory after extraction, accounting only for deliberate owner-module path changes. The source snapshot and pristine-main log provide the original body/type/attribute and test-result baseline. Run focused parser tests and verify exact literals, whole function/method bodies, struct fields, constants, original cfg guards and API visibility. Independent review must cover strict issue validation, sanitized diagnostics, delimiter agreement, CRLF asymmetry, fence byte offsets, symbol spelling and deduplication.

After shared-target permission, run cargo fmt --check, cargo check, release check/build, production clippy with warnings denied, strict 100-percent coverage and mandatory pre-push gate, plus real scoped lifecycle check and check --commit. Parent runs combined full unit/integration suites after both independent checkpoints and compares known pristine-main failure names; do not represent the focused run as a full isolated suite. Report unavailable Fledge, Attest, Ruby and Windows tooling explicitly, without installing runtimes or weakening gates.

## Observed evidence

- Pristine-main baseline: all 69 parser tests passed in the existing full unit log. Every original leaf test name is retained; owner module paths intentionally change.
- Independent source review: all 117 function/method body token sequences, including literal bytes, and all 69 complete test declarations are unchanged. Constants, types, statics, impls and original attributes are identical. The sole declaration change is the minimum `pub(super)` shared delimiter-scan seam. Facade visibility, owner-local `cfg(test)` wiring and strict-YAML/no-fallback boundaries pass review without findings. This is engineering review, not human implementation approval.
- The parser facade is 29 lines; production owners are 249, 277, 469 and 216 lines. Test files are 322, 236, 508 and 148 lines. Every result is below 1000 lines.
- Initial strict forced coverage using the existing release CLI passes all 62 specs with zero warnings and 100% file/LOC coverage. The rebuilt parser binary later passed the same final coverage gate, as recorded below.

- Focused compiled parser suite: 69 passed, 0 failed, 0 ignored; 2461 tests intentionally filtered. Compiled leaf-name inventory exactly matches the pristine 69-test baseline after the deliberate module-path changes.
- Formatting, debug check, optimized release check and production clippy with warnings denied pass. No isolated full suite or all-target clippy pass is claimed; combined full-suite baseline comparison is owned by the parent verification task.
- Fledge trust verification, Attest and Ruby were attempted and are unavailable (command not found, exit 127). Only x86_64-unknown-linux-gnu is installed; Windows execution is unavailable. No new runtime or dependency was installed and no trust gate was bypassed.
- The reviewed nine-file source checkpoint digest is `e65cff546e88cec81224b40e681d2f972fa9da939edb96871740cfa21454b7db`: SHA-256 of sorted repository-relative paths, NUL, lowercase per-file SHA-256 and LF. Source remained unchanged after review.

- Optimized release build passes. The rebuilt binary passes strict forced coverage: all 62 specs, zero warnings, 111/111 files and 149507/149507 source lines covered. The mandatory pre-push gate passes all three stages in 2 seconds.

Initial real scoped lifecycle verification passes and materializes parser spec version 14 with the approved organization section and change-log entry. The local checkpoint is produced by the real `change check --commit` flow. Human implementation review, publication, CI, acceptance and archival are not claimed.
