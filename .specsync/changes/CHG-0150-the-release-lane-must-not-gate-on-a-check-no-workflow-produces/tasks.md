# Tasks

- [x] Confirm no workflow produces `SpecSync archive binding`
- [x] Confirm the wait is unconditional and `validate` has no mode guard
- [x] Recover the deleted producer from `802ca13b^` to establish intent
- [x] Delete the wait and the embedded validation, keep the three candidate checks
- [x] Add the `dry_run` dispatch input and the `dry-run` mode
- [x] Reject unrecognized `dry_run` values instead of defaulting to promote
- [x] Verify no job consumes outputs from the deleted block
- [x] YAML parses; actionlint clean; job graph unchanged for qualify and promote
