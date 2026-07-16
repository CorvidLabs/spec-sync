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
- portable project paths reject POSIX absolute, drive-prefixed, backslash, UNC/device, control-character, dot-component, empty, and separator-ambiguous forms consistently; symlink targets apply their narrower normative rejection classes consistently;
- valid relative symlink targets containing `./`, interior `./`, repeated separators, trailing separators, or `..` remain accepted with exact bytes rather than being normalized or rejected;
- assume-unchanged paths, materialized fsmonitor-valid paths, materialized skip-worktree paths, unmerged stages, custom filters, working-tree encodings, and ident transformations fail before canonical index substitution, while absent skip-worktree files use canonical blobs and remain checkout-shape independent;
- a changed path explicitly marked fsmonitor-valid cannot be silently canonicalized from the index, including when a custom fsmonitor hook is configured;
- real ident checkout expansion is rejected before canonical substitution, while ordinary text/eol normalization remains supported;
- visibility flags and attributes on unrelated excluded paths do not block evidence for the exact governed candidate set;
- volatile paths do not block project freshness, and unrelated repository paths do not block scoped manifest or legacy acceptance input evidence, while the same flag or attribute on a relevant path still fails closed;
- broad content-conversion attributes still reject candidate regular files but do not falsely block canonical symlink targets or gitlink object IDs;
- transforming attributes reject only clean materialized tracked regular index substitution, not dirty, untracked, or sparse-absent inputs;
- Git false boolean spellings disable fsmonitor while true and pathname hooks remain fail closed;
- an attribute inventory whose combined NUL-delimited input and output exceed operating-system pipe capacity completes deterministically through bounded batches and still rejects an attributed path near the end of the inventory;
- an index mutation between evidence reads causes a bounded retry or deterministic fail-closed result instead of mixing index generations;
- a candidate worktree mutation between capture and revalidation retries or fails closed, and callers consume only captured topology/content;
- oversized candidate counts, aggregate path bytes, or attribute output fail before unbounded payload/owner work, while NUL parsing remains deterministic at the accepted boundary;
- oversized discovery streams, index/split-index dependencies, and attribute output fail before unbounded buffering;
- after positive repository detection, injected `ls-files`, `diff-files`, attribute, or malformed-index failures never downgrade to filesystem evidence and retain bounded stderr context;
- alternate `GIT_INDEX_FILE` evidence fingerprints and detects mutation of the effective index and its split dependency;
- unrelated excluded unmerged stages do not block a scoped digest, while a relevant unmerged stage fails closed;
- capped concurrent stdout/stderr drains kill and reap an overflowing child before retaining unbounded output;
- sparse-absent tracked legacy archive and baseline files remain present through canonical index bytes, while dirty archived symlink replacements use current file/missing/non-file topology;
- strict checking partitions one shared bounded evidence generation across every baseline archive without cross-subtree leakage, while an explicitly scoped real `.specsync/archive/` path retains sparse-absent tracked entries;
- strict checking evaluates all verifying records against one stable project-input snapshot and reports one shared baseline-authority failure instead of repeating it for every legacy archive;
- deleting a previously bound authority baseline makes definition reapproval fail closed instead of retaining the stale baseline digest;
- first approval of an authority covering a missing baseline fails instead of producing an unbound definition, while unrelated changes still no-op;
- clean tracked executable definition artifacts remain valid regular files while symlink/gitlink/non-file topology fails closed;
- a modified tracked symlink replaced by a regular file signs file topology/current bytes, while deletion signs missing and a clean materialized symlink retains canonical symlink topology;
- definition artifact caps bind canonical clean bytes and current dirty/untracked bytes rather than host-smudged metadata length;
- selected definition artifacts reject clean tracked, dirty, and untracked symlinks before reading any internal or external target, independently of referent size;
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
- all 44 standalone archives present at exact released-main cutoff `fc6e70bccd5af61043183e247f37b1f9a9b92247` authenticate only through a strictly sorted unique baseline ledger bound into CHG43's definition and, after acceptance, its manifest-backed closing and trusted acceptance/history anchor;
- the pre-accept bootstrap passes only with exact ledger bytes, definition-bound ledger digest, valid authority definition approval, and a canonical cutoff equal to the authority base and ancestral to current history; after authority acceptance the same corpus requires manifest-backed closing/history authority and cannot downgrade to bootstrap;
- downgrade/unknown schema, wrong authority, stale ledger bytes, unavailable objects, arbitrary ancestor/descendant/divergent or otherwise mismatched cutoff, post-cutoff/zero/multiple introductions, unsorted/duplicate entries, and modern manifest archives without snapshots never use the baseline fallback;
- editing, adding, deleting, chmodding, symlinking, gitlinking, or replacing any baseline archive subtree entry fails historical integrity;
- an archive introduced after the exact cutoff is absent from the closed baseline inventory and fails closed rather than inheriting legacy trust;
- standalone baseline authentication cannot satisfy active accepted current validity, archive preflight candidate validity, or mask a predecessor as an archived successor candidate;
- active accepted current-input drift remains a strict-check failure;
- check and status remain green immediately after an uncommitted archive move through location-aware artifact reads and the authenticated accepted snapshot;
- duplicate active/archive IDs and ambiguous dated archive locations fail closed;
- old state and verification JSON plus definition and closing digest bytes remain unchanged when new fields are empty or absent;
- one explicitly requested supported approval command atomically emits an adjacent marked current/full then 5.0.1-projection definition pair from one snapshot, preserves prior approvals/reopenings, and both the current engine and immutable 5.0.1 engine accept their respective contract;
- portable-pair validation rejects orphaned members, wrong actors or timestamps, reversed roles/order, wrong current or projection digests, same digests, intervening gates, wrong pair/schema/projection/change/correction metadata, duplicate/replayed members, unsupported nonempty post-5.0.1 fields, and stale/reverted definitions instead of searching historical approvals;
- golden projection fixtures cover active and archived state, omitted/default and explicit-false representation, and unsupported corrections; the normal LF pair passes the actual immutable 5.0.1 engine, while a forced clean CRLF-smudged artifact is rejected with its exact path before any ledger mutation;
- strict project checking reports an invalid required definition approval for Approved, Implementing, and Verifying records while Draft records remain exempt;
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
