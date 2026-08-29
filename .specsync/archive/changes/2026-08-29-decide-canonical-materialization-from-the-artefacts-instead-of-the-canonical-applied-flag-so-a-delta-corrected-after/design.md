---
change: decide-canonical-materialization-from-the-artefacts-instead-of-the-canonical-applied-flag-so-a-delta-corrected-after
artifact: design
---

# Design

## The shape

`materialize_change_deltas` used to read:

```rust
ensure_approved_delta_bodies_unchanged(root, &record)?;   // does the delta match its approval?
...
if record.canonical_applied { return Ok(record); }        // <- everything below is skipped
if is_ci_project(root) { return Err(...); }
let mut prepared = prepare_delta_application(root, &record)?;
```

It now reads:

```rust
ensure_approved_delta_bodies_unchanged(root, &record)?;   // does the delta match its approval?
...
let prepared = prepare_pending_delta_application(root, &record)?;   // does the tree match the delta?
if record.canonical_applied && prepared.pending.is_empty() { return Ok(record); }
if is_ci_project(root) { return Err(... names prepared.pending ...); }
let mut prepared = prepared.files;
```

Preparation writes nothing; it returns `(files, pending)`. `pending` is empty exactly when every
affected module's canonical outputs are already current, so the short-circuit survives intact for
the case it was built for and only that case.

`accept_change_with_gate` gets the same replacement, because acceptance is the second place deltas
reach the canonical spec and had the identical `if canonical_applied { Vec::new() }` hole. An
already-current tree yields an empty `files`, so the acceptance manifest it feeds is byte-identical
to what the old branch produced.

## The evidence, per output

Materialization produces three outputs per module. The short-circuit skipped all three, because
`bump_spec_version` and `append_changelog` have exactly one caller and it sits below the flag.

| output | evidence that it is current |
|---|---|
| delta applied to the canonical files | every item the delta declares is already reflected (`delta_item_is_applied`) |
| the spec's `version:` bump | the Change Log names this change (`changelog_records_change`) |
| the spec's Change Log row | the Change Log names this change (`changelog_records_change`) |

The bump and the row share one piece of evidence deliberately. They are written by the same two
adjacent lines, exactly once per (change, module); a bumped `version:` leaves nothing behind naming
the change that bumped it, so the integer alone cannot distinguish "this change bumped it" from
"a later change did". Keying both on the row also makes re-materialization of a CORRECTED delta
refresh the text without a second bump and a second row — one change bumps one module's version
exactly once.

## Two edges that shaped it

**`prepare_delta_application` was not idempotent.** `apply_markdown_block` refuses to remove a
block that is not there. That refusal is right on a first run: it means the delta names something
that never existed. Re-running the applier over an already-materialized `## REMOVED` delta would
therefore have converted the silent skip into a hard error. The fix is not to weaken the applier —
`delta_item_is_applied` asks the question the applier cannot answer, because the applier applies —
and the resulting convergence is scoped to `record.canonical_applied`. A first materialization
still fires every refusal it ever did; only afterwards may "already reflected" read as done,
because only then is "absent" a state this change itself produced.

**The record is hashed.** `ChangeRecord` is serialized into `definition_digest` in four places
(`definition_digest_for_correction_count`, `legacy_task_definition_digest_for_correction_count`,
`historical_definition_digest_matches`, `portable_definition_digest_pair_v501`), each normalizing
`canonical_applied`, `correction_count` and `updated_at` out by hand — one of them by splicing
`,"canonical_applied":false` into the serialized bytes to reproduce an older writer. Adding a
`materialized_delta_digests` field would have moved every one of those digests on any live record
unless normalized out in all four. Deriving the answer from the artefacts persists nothing, so no
archived ledger changes shape and no digest moves.

## Refactor carried along

`markdown_block_range` is extracted from `apply_markdown_block`, so that applying an item and
asking whether an item is already applied read the same block out of the same scan. Two answers
derived from two scans is how a tree and the record of it drift apart, which is the subject of this
change.
