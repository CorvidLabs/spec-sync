---
change: close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first
artifact: tasks
---

# Tasks

- [x] Red: the tests for W1, S1, S2, S3, S5, L2, X1, X2, N2, H1 and C1 to C5 fail on 6.4.0, and the non-Git spawn scan is added red. Evidence: `e334bd8a`; the non-Git spawn scan listed nine sites before they were marked.
- [x] State folders: the journal in the user's state folder (S5); N2; L2; the journal records its project root and other journals are refused (S2). Evidence: `25aef628`, `ba597f66`, `2639d058`.
- [x] The read cache through `LocalState` (C1). Evidence: `ba597f66`.
- [x] Hooks: the common Git directory as a base (H1), resolved comparisons (X2), no outside folder without the person's own config (S1), and a clean `rev-parse` environment. Evidence: `25aef628`.
- [x] Listings through `read_project_dir` (W1); the scans widened and every site routed or marked (X1). Evidence: `ba597f66`; the read, write, Git and spawn scans pass.
- [x] `read_typed_file` for `--manifest` and `--template` (S3). Evidence: `25aef628`.
- [x] Refused spec reads reported in `check --only-status`, `resolve` and `stale` (C2 to C4); `save` refuses a cache over the bound (C5). Evidence: `25aef628`.
- [x] The five history clauses removed; CHANGELOG, CLI page and HLD updated. Evidence: the materialized MODIFIED deltas; CHANGELOG, `cli.md` and `HLD.md` in `ba597f66`.
- [x] Green on macOS (verify lane) and Linux, through one-lane; `change audit` passes. Evidence: at `2639d058`, the macOS verify lane passed all 10 steps, and Linux (`rust:1.89-bookworm`, non-root) has clippy clean with 3162 unit and 741 integration tests passing.
