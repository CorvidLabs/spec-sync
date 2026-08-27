---
change: audit-every-module-context-md-against-the-source-and-correct-every-false-or-stale-claim
artifact: design
---

# Design

## The editing rule

A correction replaces a false statement with the true one and stops. It does not restructure the
paragraph, does not add evidence-citation scaffolding, and does not rewrite judgement that happens
to sit next to a wrong fact.

Where a claim was true when written and a later change falsified it, the correction preserves the
history rather than deleting it — `check_project_quiet` is named as removed by #543, the CI
lifecycle workflows as deleted by #499, `auto_regen_stale_specs` as removed with embedded
inference in #335. A reader who followed the old lesson needs to know it was true once and what
replaced it; silently swapping the text leaves them unable to reconcile what they remember.

## Three shapes of defect, and how each was handled

**Dead pointer.** A named symbol, path, or test that does not exist. Replaced with the real name.
This is the cheapest class to find and the least dangerous — a broken pointer is visible.

**Decayed count.** A number that was right and drifted. Replaced with a measured value and the
counting basis, so the next reader can tell what denominator it uses. Where the count is anchored
to an issue and phrased in the past tense (`0 of 107 archived reviews (#694)`), it was left: it is
a record of a measurement, not a claim about today.

**Reversed behaviour.** The dangerous class, because every symbol in the sentence still exists.
`Nothing writes it any more` names a real file, uses correct vocabulary, and is false;
`floor_sequence_ledger_to_committed` writes it from inside every lifecycle commit. Finding these
required reading the function, not grepping the name.

## Counts that decay, and why they were not simply deleted

Several corrected numbers will drift again — 2263 tracked `.md` files, 188 of 202 ledgers, 2407
unit tests. Removing them would stop the decay and also remove the only thing that makes the
surrounding claim checkable. A wrong number is worse than a decaying one: it is confidently
precise. Each corrected count now carries its counting basis in the sentence, so the next reader
can re-run it instead of trusting it, and `research.md` records the command.

## Where the loop's own residual sits

`21 of 39 call sites normalize` is the corrected figure from #714, and no command recorded
anywhere reproduces it exactly (see `research.md`). It was not changed, because three defensible
methods give 21, 22 and 24 and none shows it wrong. That is the honest state of it — and it is the
best available argument for #714's option 1: a lesson that cites its command is re-derivable, and
a lesson that cites only its conclusion is not, even when the conclusion is right.

## Scoping

`--spec` for all 34 owning modules AND `--no-spec-change`, which `docs/ADOPTING.md` documents as
the combination for "these modules own the paths, but no canonical spec text moves". `--spec`
alone would make `validate_delta_files` demand one semantic delta per affected spec, and a delta
rewrites the canonical `<module>.spec.md` this change does not touch. Dropping `--spec` (the
fold-back recipe in `ADOPTING.md`) would leave 34 edited modules with no declared owner; this is a
correction audit, not a fold-back of an archived bundle, so that recipe's rationale would not be
true of it.
