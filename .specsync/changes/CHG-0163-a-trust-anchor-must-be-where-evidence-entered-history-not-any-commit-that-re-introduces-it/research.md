# Research

## The defect

`authenticated_accepted_transition` resolves the trusted transition for an archived change by
running `git log --diff-filter=A -- <archive-dir>/accepted-state.json` and accepting any anchor
whose committed `accepted-state.json` / `verification.json` / `approvals.json` are byte-identical
to the working tree. No cutoff, no ancestry bound, no ordering rule.

The check is circular: it authenticates working-tree bytes against a commit that *contains those
bytes*. Any commit of the current state qualifies. `--diff-filter=A` is the only thing keeping
that from being trivially true — a tampering commit is a modify, so it is never an anchor — and
re-introducing the package produces an addition.

Reproduced against `origin/main`, three shapes:

```
tamper-then-relocate:              expected refusal, got anchor 2746d148
tamper-and-relocate-in-one-commit: expected refusal, got anchor 0e53b51a
forged-reopen-and-re-archive:      expected refusal, got anchor 6360b7f4
```

The third involves **no rename at all**. `reopen` moves a package to `.specsync/changes/<id>/`
and `archive` moves it back, so an attacker who tampers in between produces a fresh introduction
at a path SpecSync itself writes. The original diagnosis — "renaming an archive directory
launders tampering" — was too narrow, and a fix bounding only the archive path would have left
this open.

## What the corpus is, measured before choosing a rule

161 archives at `65755ac7`:

| dimension | counts |
|---|---|
| routes through `authenticated_accepted_transition` | 117 |
| routes through `authenticate_legacy_archive_baseline` | 44 |
| `validity` today | 154 authenticated, 7 corrupt |

Anchor sources across the 117 — the number that constrains any fix:

| stage | eligible for | won |
|---|---|---|
| A `accepted_transition_anchors` (active path) | 19 | 19 |
| B archived `--diff-filter=A` | **117 (all)** | 98 |
| C `accepted_recording_anchors` fallback | **0 — never executes** | 0 |
| D working-tree closing evidence | 0 | 0 |

Two findings that reframe the problem:

- **Stage C is dead code for this corpus.** Its comment claims it exists so squash-merged
  archives stay authenticated; stage B is actually doing that, because the archive directory is
  added by the squashed commit and so the addition survives a squash trivially.
- **Stage B cannot be narrowed away.** 90 of 117 have *only* stage B. Forcing stage C to
  evaluate anyway finds an anchor for just 27 of 117; for 90 it searches zero commits, because
  their active `state.json` never recorded `accepted` in committed history at all — `finalize`
  accepts and archives in one uncommitted step.

An ancestry bound would cost nothing (117/117 winning anchors are already ancestors of HEAD) and
buy nothing, because the laundering commit is an ancestor too.
