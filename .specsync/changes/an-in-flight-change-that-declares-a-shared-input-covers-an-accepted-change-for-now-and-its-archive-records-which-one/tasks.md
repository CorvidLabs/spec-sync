---
change: an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now-and-its-archive-records-which-one
artifact: tasks
---

# Tasks

- [x] Failing tests in `src/change_tests/provisional_coverage.rs` and an integration test through the binary
- [x] The in-flight coverer and `provisionally-covered` evidence
- [x] Audit warning; doctor, successor, retirement and reopen decisions
- [x] `coverage.json` written by `change archive` and `change finalize`, read by status, show and `audit --archives`
- [x] #688's sentence in each of the two archive refusals
- [x] Docs and CHANGELOG
- [x] Released 6.3.0 reads an archive holding `coverage.json` as before
- [x] `fledge lanes run verify`, `change audit --strict` and the pre-push gate
