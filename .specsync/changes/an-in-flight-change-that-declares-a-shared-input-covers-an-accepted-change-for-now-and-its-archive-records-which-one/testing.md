---
change: an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now-and-its-archive-records-which-one
artifact: testing
---

# Testing

`src/change_tests/provisional_coverage.rs`, failing first:

- An accepted change, a later approved change declaring the same module and path, and a committed edit of the file: `provisionally-covered` naming the later change; audit has no error and one warning naming both; doctor lists no drift.
- The same edit with no declaring change: stale, and audit fails as before. A draft, or a change declaring the path without the owning module, does not cover.
- A provisionally covered change is not taken as a covering successor or a retirement replacement; `change reopen` accepts it.
- When the covering change is accepted, the earlier change reads `successor-covered`.
- The archive writes `coverage.json` naming the file and the covering change, through the literal `change archive` on a workflow-v1 fixture and through `change finalize` on workflow v2; another change's finalize is not refused because of it.
- After the archive, with the covering change sent back: the archive reads `authenticated-history`; status and show print the entry and the warning; `audit --archives` warns and exits 0. With the coverer accepted or archived, no warning. An archive with no `coverage.json` reads as before.
- REQ-change-168: both refusals name the other change and say it is blocked too, both still name `change reopen`, and the reopen frees both; with no later covering scope the first refusal reads as before.

Through the binary: `change audit --strict` exits 0 with the warning, the two refusals and the reopen that frees both. Released 6.3.0 against an archive holding `coverage.json`: status, audit, check and doctor clean. Full lane.
