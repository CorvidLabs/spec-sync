---
change: CHG-0043-make-accepted-change-validity-successor-aware-with-exact-per-input-evidence-rec
artifact: testing
---

# Testing

Focused unit and integration regressions will prove:

- an exact manifest and an exact legacy aggregate remain valid;
- one terminal successor covers one changed entry;
- two terminal successors may cover disjoint changed entries;
- partial coverage leaves the predecessor stale;
- draft, approved, implementing, verifying, failed, stale, tampered, no-spec, and semantically empty successors never mask a stale predecessor;
- a path-only successor plus a different module-only successor fails the cross-product attack;
- a multi-owner entry requires one exact path/module obligation per owner and cannot pass through owner intersection;
- ID order, timestamps, and scope overlap without an exact signed or historically reconstructed semantic transition never satisfy succession;
- an acceptance commit whose immediate parent already contains implementation bytes cannot fabricate succession; the trusted signed base tree supplies old bytes and the descendant accepted-transition tree supplies new bytes;
- acknowledged same-sequence predecessor IDs sort deterministically by full ID after numeric sequence;
- deletion, file/executable mode, symlink target, gitlink, missing, and non-file changes remain distinct;
- clean tracked inputs hash canonical index blob bytes across LF and CRLF-smudged checkouts, while dirty tracked and untracked inputs continue to hash their current working-tree bytes;
- tracked symlinks hash their canonical Git target payload even when the host materializes them as ordinary files;
- deleting a tracked symlink yields a deterministic missing/mode-zero entry in project freshness, manifest acceptance, and legacy acceptance instead of reading the historical target or failing on the absent path;
- portable path and symlink-target validation returns the same result on every host for POSIX absolute, drive-prefixed, backslash, UNC/device, control-character, dot-component, empty, and valid relative forms;
- same-payload mode and kind transitions produce different `specsync.acceptance-entry.v1` full-entry digests and require exact succession tuples;
- CHG43's governed integration test path is classified exact-only, accepts without invented module ownership, and requires reopen after later modification;
- recursive successor chains validate and cycles fail closed deterministically;
- legacy historical reconstruction succeeds only when a trusted accepted Git tree reproduces the signed aggregate exactly;
- ambiguous, unavailable, uncommitted, or mismatching legacy reconstruction fails closed;
- multiple trusted commits with identical deduplicated historical evidence remain valid while distinct reconstructed evidence fails as ambiguous;
- active accepted check, status, reopen, and archive eligibility agree for exact, successor-covered, and stale inputs;
- an unrelated authenticated archive remains strict-check green after its exact-only inputs evolve, while tampered archived state, snapshot, definition, verification, manifest, closing evidence, or succession tuples fail historical integrity globally;
- archived status JSON reports `authenticated-history` or `corrupt-history` separately from active accepted exact/successor/stale validity;
- a stale or unprovable archived candidate cannot mask an active accepted predecessor, while a valid archived successor whose own changed inputs are recursively covered remains usable;
- historical-integrity cache results are never reused as recursive candidate-valid results;
- archive preflight authenticates the target and recursively validates active accepted roots and dependent candidates without blocking on unrelated archive drift;
- a provable legacy archive passes historical integrity and an unverifiable legacy archive fails closed;
- all 43 standalone archives present at exact release-branch cutoff `7739ea17b067ef636c106ceca6bcf59eee8e6213` authenticate only through a strictly sorted unique baseline ledger bound into CHG43's definition and, after acceptance, its manifest-backed closing and trusted acceptance/history anchor;
- the pre-accept bootstrap passes only with exact ledger bytes, definition-bound ledger digest, valid authority definition approval, and a canonical cutoff equal to the authority base and ancestral to current history; after authority acceptance the same corpus requires manifest-backed closing/history authority and cannot downgrade to bootstrap;
- downgrade/unknown schema, wrong authority, stale ledger bytes, unavailable objects, arbitrary ancestor/descendant/divergent or otherwise mismatched cutoff, post-cutoff/zero/multiple introductions, unsorted/duplicate entries, and modern manifest archives without snapshots never use the baseline fallback;
- editing, adding, deleting, chmodding, symlinking, gitlinking, or replacing any baseline archive subtree entry fails historical integrity;
- an archive introduced after the exact cutoff is absent from the closed baseline inventory and fails closed rather than inheriting legacy trust;
- standalone baseline authentication cannot satisfy active accepted current validity, archive preflight candidate validity, or mask a predecessor as an archived successor candidate;
- active accepted current-input drift remains a strict-check failure;
- check and status remain green immediately after an uncommitted archive move through location-aware artifact reads and the authenticated accepted snapshot;
- duplicate active/archive IDs and ambiguous dated archive locations fail closed;
- old state and verification JSON plus definition and closing digest bytes remain unchanged when new fields are empty or absent;
- malformed, oversized, unsorted, duplicate, conflicting, unapproved, and non-portable succession tuples fail closed, while exact one-to-one approved tuples pass;
- unrelated accepted changes cannot satisfy coverage and numeric sequence ordering handles IDs wider than four digits.

Canonical requirement evidence covers `REQ-change-012`, `REQ-change-014`, `REQ-change-017`, `REQ-change-018`, `REQ-change-020`, `REQ-change-024`, `REQ-change-032`, `REQ-cmd-change-001`, and `REQ-cli-args-001` through the focused regressions above.

Verification commands:

- focused `cargo test` filters for manifest, successor, legacy, reopen, status, and archive behavior;
- the eight Windows regressions run with `core.autocrlf=true` so canonical tree bytes, transition reconstruction, project freshness, and archive retry/restore remain host-independent;
- `fledge lanes run verify`;
- `specsync check --strict --require-coverage 100 --force`;
- `specsync agents status`;
- `fledge trust doctor` and `fledge trust verify`;
- `git diff --check` and unfinished-artifact scans.
