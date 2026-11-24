---
change: an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now-and-its-archive-records-which-one
artifact: research
---

# Research

- Coverage was filtered to accepted or archived successors (`successor_covers_input`).
- Four callers took any `Ok` validity as settled: successor candidates, closing-approval validity, the reopen refusal and retirement replacements; each is decided in the design.
- The archive preflight re-checks other accepted changes and already passes a provisional one; the second refusal site refuses any archive while any accepted change is stale, and does not simulate that this archive is what makes it stale.
- On workflow v2, `change archive` refuses ("uses same-PR finalization; run specsync change finalize"); real projects archive through `change finalize`. Both write `coverage.json` and both are tested (ss-lead's build note of 2026-11-19).
- Released 6.3.0 on an archive holding `coverage.json` (definition time, throwaway project): status, JSON status, `audit --strict`, `audit --strict --archives`, `check --strict`, list and doctor all exit 0, archived and `authenticated-history`.
