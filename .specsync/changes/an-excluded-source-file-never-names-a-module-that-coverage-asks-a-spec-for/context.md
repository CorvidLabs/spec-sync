---
change: an-excluded-source-file-never-names-a-module-that-coverage-asks-a-spec-for
artifact: context
---

# Context

Issue #613. The flat-file-stem block in coverage (`src/validator/tests.rs`, included into the
validator) iterates `source_snapshot.files` before `exclude_patterns` filtering, while the
coverage denominator (`all_source_files`) is filtered. So a file the user excluded still
contributed its stem as a module name, and coverage then reported that module as lacking a spec.
Leif decided (orc relay, 2026-11-06, recorded on #613): excluded means ignored, so an excluded
source file never names a module and never asks for a spec. The configuration docs call
`exclude_patterns` "additive source exclusions". The fix applies `coverage_path_is_excluded` in
that block, as the denominator does. No requirement text changes.
