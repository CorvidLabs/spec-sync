---
change: CHG-0043-make-accepted-change-validity-successor-aware-with-exact-per-input-evidence-rec
artifact: requirements
---

# Requirements

## REQ-CHG-0043-001 — Exact signed acceptance inputs

SpecSync SHALL persist a domain-separated and length-framed manifest entry for every acceptance input.

Acceptance criteria:

- Every entry binds normalized repository-relative path, entry kind, Git mode, exact content or symlink-target payload digest, a `specsync.acceptance-entry.v1` full-entry topology digest, and deterministic owners from the signed post-delta snapshot.
- Missing, regular-file, executable-file, symlink, gitlink, and non-file topology remain distinguishable.
- Manifest schema, ordering, uniqueness, kind/mode combinations, digest syntax, entry/path/owner bounds, portable symlink bytes, exact gitlink object IDs, and aggregate reproduction are validated fail closed.
- Candidate-scoped Git evidence accepts at most 100,000 paths, 4,096 bytes per path, and 64 MiB of aggregate path bytes before payload, owner, or attribute work; bounded NUL-safe attribute batches reject active regular-file `filter`, `working-tree-encoding`, or `ident` conversion without blocking unrelated, symlink, or gitlink paths.
- Canonical index substitution rejects evidence-relevant assume-unchanged, materialized fsmonitor-valid, materialized skip-worktree, and unmerged paths; absent sparse paths use canonical index topology, and the complete read retries or fails closed if the index or split-index generation changes.
- Recognized governed test/fixture paths and delivery metadata receive reserved exact-only owners; unmapped production source inputs fail acceptance instead of inheriting every affected module.
- Empty supersedes and absent verification fields are serde-defaulted and omitted so legacy state JSON, verification JSON, definition digests, and closing digests remain byte-identical.
- New manifests use `specsync.acceptance-manifest.v1`; legacy raw-content aggregates retain `specsync.acceptance-input.v2`.
- Exact current inputs validate without successor inference.

## REQ-CHG-0043-002 — Same-successor semantic coverage

SpecSync SHALL accept changed historical inputs only through terminal semantic successors that bind path and module in the same successor record.

Acceptance criteria:

- Multiple terminal successors may collectively cover different changed entries.
- A successor definition contains a durable digest-bound `supersedes` edge naming the predecessor and one explicit path/module/old-full-entry-digest intent for every adopted obligation.
- For every changed entry and every canonical owner, at least one same successor signs predecessor ID, path, that owner module, predecessor full-entry digest, and successor full-entry digest.
- The closing binding is generated only from an approved edge, a semantic delta for that module, and the exact transition from a trusted definition-signed base commit to its descendant unique accepted-transition anchor; the acceptance commit's immediate parent, ID order, timestamps, and scope overlap are not succession evidence.
- Succession evidence uses `specsync.semantic-succession.v1` and validates schema, bounds, strict `(numeric sequence, full predecessor ID, path, module)` order and uniqueness, portable paths, canonical modules, lowercase full-entry digests, conflict rejection, and exact one-to-one approved-obligation derivation.
- Full-entry topology binding distinguishes same-payload chmod, file/symlink, missing/non-file, and gitlink transitions.
- A path-only successor plus an unrelated module-only successor never satisfies one entry.
- No-spec, draft, approved, implementing, verifying, failed, stale, tampered, and semantically empty candidates never mask predecessor failures.

## REQ-CHG-0043-003 — Recursive fail-closed validity

Successor validity SHALL be recursive, cycle-safe, and shared by every lifecycle caller.

Acceptance criteria:

- Accepted and safely archived candidates require valid definition, verification, closing approval, semantic delta, history integration, and exact-or-successor-covered inputs.
- A visiting-set cycle fails closed; memoized completed results avoid redundant graph validation.
- Active accepted status, reopen, and archive eligibility use the same recursive current-input validator; archived status and strict checking use a separate historical-integrity conclusion.
- Reopen rejects evidence that is exact or validly successor-covered and accepts only genuinely stale accepted evidence after its existing actor, reason, and audit checks.

## REQ-CHG-0043-004 — Fail-closed legacy and archive compatibility

SpecSync SHALL preserve compatible accepted history without inventing evidence.

Acceptance criteria:

- A legacy record whose current aggregate equals its signed aggregate remains valid byte-for-byte.
- A stale legacy record is reconstructable only from its unique trusted accepted-transition anchor and only when the reconstructed historical manifest reproduces its signed aggregate exactly; later trusted commits yielding identical deduplicated evidence do not create ambiguity.
- A legacy successor is usable only when its trusted definition-signed base tree and descendant unique accepted-transition tree prove the same old-entry-to-new-entry transition and semantic module delta required of new signed succession evidence.
- Ambiguous historical snapshots, uncommitted acceptance inputs, missing objects, or aggregate mismatch fail closed.
- Every archive authenticates one accepted projection from a trusted transition, its definition, verification, manifest or uniquely reconstructed legacy aggregate, closing approval, canonical succession tuples, and byte-identical accepted-state snapshot without comparing signed inputs to today's workspace.
- A committed `.specsync/archive/legacy-baseline.json` v1 ledger authenticates enumerated pre-CHG43 standalone archives through an explicit two-phase authority. Before authority acceptance, the authority record must carry the exact domain-separated ledger digest in its definition, have a valid definition approval, and use a cutoff exactly equal to its definition-bound base commit and ancestral to current history. After authority acceptance, the authority must additionally be manifest-backed and closing-valid with the exact-owned ledger path in its signed manifest.
- The cutoff must be one canonical 40-hex commit equal to the human-approved authority base and ancestral to both current history and the authority change's accepted/history anchor; arbitrary earlier, later, or divergent commits fail closed. Every introduction must be uniquely reachable at or before the cutoff and must introduce the subtree absent from every parent. Each current subtree must exactly reproduce ledger inventory, paths, bytes, Git modes and kinds, including portable symlink targets; extra/missing entries, gitlinks, non-files, downgrade/unsorted/duplicate entries, post-cutoff anchors, unavailable trust roots, or digest mismatch fail closed.
- Baseline authentication yields historical integrity only and is never accepted-transition, current-input, semantic-succession, candidate, preflight, reopen, or closing evidence. Modern manifest archives never use the ledger fallback.
- Archived status reports `authenticated-history` or `corrupt-history`; active accepted status alone reports `exact`, `successor-covered`, or `stale`.
- An archived record selected to cover an active accepted obligation additionally enters recursive current-input validation; stale or unverifiable archived candidates never mask the active predecessor, while recursively covered archived candidates remain usable.
- Archive preserves accepted-state bytes authenticated by the unique trusted accepted-transition anchor and prior human closing evidence, resolves every artifact relative to the discovered archive location, rejects duplicate locations, and preflights target historical integrity plus every active accepted root and dependent candidate before mutation; unrelated current-input drift in another authenticated archive does not block the move.
- Legacy archive and baseline snapshots include sparse-absent tracked entries from canonical index evidence, classify dirty tracked symlinks from current file, symlink, missing, or non-file topology, and reject a dirty or untracked missing authority baseline without retaining a prior definition binding.
- Archive never mints approval, post-move failure restores byte-identical source artifacts without residue, and unverifiable legacy archives fail closed.
