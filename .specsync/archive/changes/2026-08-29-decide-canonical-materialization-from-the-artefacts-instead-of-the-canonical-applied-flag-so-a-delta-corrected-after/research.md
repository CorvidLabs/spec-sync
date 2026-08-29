---
change: decide-canonical-materialization-from-the-artefacts-instead-of-the-canonical-applied-flag-so-a-delta-corrected-after
artifact: research
---

# Research

## The three fix directions, and the evidence for choosing among them

Issue #741 offered three. Each was checked against the code rather than judged on description.

### (1) Record `materialized_delta_digests` and clear the flag when it moves

The issue's preferred direction. **Rejected on measured cost, and on insufficiency.**

`ChangeRecord` is not merely persisted — it is serialized and hashed. `grep -n "canonical_record\|canonical\.canonical_applied" src/change.rs`
finds four sites that clone the record, force `state = Draft`, `canonical_applied = false`,
`correction_count`, and `updated_at = 0`, then hash the bytes:

- `definition_digest_for_correction_count`
- `legacy_task_definition_digest_for_correction_count`
- `historical_definition_digest_matches`
- `portable_definition_digest_pair_v501`

Three of them additionally splice `,"canonical_applied":false` into the serialized bytes to
reproduce what an older writer emitted. A new field that is `Some(..)` on a live record would move
every definition digest computed from that record unless it were normalized out in all four, and
the splicing already there shows what that compatibility surface costs.

It is also **not sufficient on its own**, which the issue's own comment says: the version bump and
the Change Log row are not derivable from a delta digest, so the flag would have to mean
"materialization completed for this delta content" — a claim a digest cannot fully carry.

### (2) Ask the second question above the short-circuit

**Taken, minus the refusal.** The issue notes materialization "may not be invertible". It is not
invertible, but it does not need to be: the question is not "what was the spec before?" but "does
the spec already carry what the delta declares?", which is decidable per item.

- `## ADDED` / `## MODIFIED`: the block is present and `markdown_block_matches` says its content
  equals the declared content. `apply_markdown_block` already relies on this exact predicate for
  its ADDED convergence arm, with the comment *"Re-deriving the canonical tree must converge"*.
- `## REMOVED`: the block is absent.

The version bump and the Change Log row are not decidable this way, so they get their own
evidence: the row itself. Refusing rather than repairing was rejected — it would have made the
ordinary review-correct-reapprove loop terminate in an error, which is direction (3) wearing a
different message.

### (3) Refuse re-approval once `canonical_applied` is set

**Rejected.** #542 already gives this lifecycle one dead end; the ordinary act of correcting a
delta after review should not be a second.

## The trap under the fix

`grep -n "cannot modify/remove missing block" src/change.rs` — `apply_markdown_block` refuses a
`## REMOVED` whose block is absent. Re-running the applier over an already-materialized removal
therefore ERRORS. Any "re-materialize when stale" fix that does not separate "already applied" from
"cannot be applied" turns #741's silent skip into a hard failure on every corrected `## REMOVED`
delta. The refusal itself is worth keeping for a first run, so the reading is scoped rather than
weakened.

## Measurements

- **208** `state.json` under `.specsync/` (193 with `canonical_applied: true`) and **208**
  `approvals.json` deserialize into `ChangeRecord` / `ApprovalLedger` under the changed binary and
  round-trip. No field was added, so nothing about the persisted shape moved; this was run rather
  than reasoned about, per the standing rule from #672 / #684 / #719.
- **446 of 454** archived (change, module) pairs carry a Change Log row naming the change in that
  module's current spec, which is what makes the row usable as evidence that the bump and row were
  written. The eight exceptions — `CHG-0003` (`ai`), `CHG-0068` (`cmd_comment`, `cmd_coverage`,
  `cmd_generate`), `CHG-0069` (`cmd_agents`, `cli_args`), `CHG-0088` (`github`), `CHG-0103`
  (`change`) — are all pre-6.0, archived, and unreachable from `check`, which runs only over
  approved / implementing / verifying changes.

## Prior art in this binding

#711 recorded delta bodies against their approval. #719 stopped a later approval withdrawing that
record. #730 narrowed the digest so a checkout's line-ending rewrite could not invalidate an
approval. This is the same binding failing in the opposite direction — the record updates while the
artefact it governs does not — and it is this release's recurring shape once more: a question that
cannot be answered is not asked, and its silence reads as a pass (#672, #684, #689, #720, #728).
