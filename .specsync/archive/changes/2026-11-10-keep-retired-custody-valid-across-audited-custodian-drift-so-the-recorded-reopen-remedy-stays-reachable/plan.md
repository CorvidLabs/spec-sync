---
change: keep-retired-custody-valid-across-audited-custodian-drift-so-the-recorded-reopen-remedy-stays-reachable
artifact: plan
---

# Plan

## Approach

Retirement validation currently re-runs the custodian's live delivery-input freshness and
re-derives custody classifications from the custodian's *current* evidence on every roster
load. That makes a retired record invalid the moment its custodian legitimately drifts, and
because every command loads the roster, the prescribed remedy (`change reopen <custodian>`)
aborts on the same check it exists to fix (#1021). It also retroactively refuses committed
receipts whose recorded classifications predate the `@exact:*` byte-equality rule (#1018),
because re-derivation applies today's rules to historical receipts.

Split the two concerns:

- Retire time (`retire_change*`) keeps the full strength: a genuinely accepted, fresh
  custodian, custody recomputed from authenticated current evidence, classifications derived
  under current rules.
- Validation time (`validate_retirement` and the roster load) stops re-running custodian
  freshness and stops re-deriving classifications. It authenticates the custodian's
  *recorded* accepted evidence and checks that custody still holds: every receipt custody
  entry names a frozen prior-manifest input (digest and owners must match), the recorded
  classification is consistent with the receipt's own fields, and the custodian's current
  authenticated manifest still carries the path under the recorded replacement owners.
  Digest evolution under unchanged signed owners is the custodian's audited lifecycle, not
  a custody break. Custodian staleness stays visible where the codebase already reports it:
  as terminal-evidence data on the custodian's own record.

## Out of scope

- Custody transfer to different owners after retirement (including `@exact:*` to named-owner
  migration). A custody path whose owners change still refuses; that needs its own design.
- The frozen-canonical conflict a reopened change can hit at re-verification (#1018 already
  ships the retirement route for that stage).
- Retirement chains (a custodian that is itself retired) still refuse.

## Steps

1. Add `retirement_custody_holds` in `src/change/retirement.rs`: authenticate the
   custodian's recorded evidence, then check predecessor binding, receipt-internal
   classification consistency, and current path/owner custody per entry.
2. Call it from `validate_retirement_with_records` in place of the
   `retirement_custody` recompute and the closing/manifest digest equality checks.
3. Regression test in `src/change_tests/recovery_retirement.rs` reproducing the site shape:
   retire, drift a custody input, assert validation holds and `reopen` succeeds, complete
   re-verification/review/finalize, assert custody still holds with evolved digests; assert
   owner tampering still refuses.
4. Update `specs/change/change.spec.md` (Public API description of `validate_retirement`,
   changelog, version) and record the lesson in `specs/change/change.3md`.

## Risks

- A custody path dropped from the custodian's manifest (file removed) still refuses
  validation with no automated remedy; acceptable, because dropping a retained obligation is
  exactly what custody exists to catch.
- The receipt's recorded `replacement_closing_digest`/`replacement_manifest_digest` become
  provenance rather than an equality gate; the predecessor binding against the frozen,
  digest-pinned prior manifest keeps forged custody claims refused.

## Constraints consulted

- Terminal-evidence staleness for accepted/archived records is reported as data
  (`TerminalEvidenceSummary::Stale`), never a load error; retired records now follow the
  same pattern for their custodian's freshness.
- Receipts are immutable and pinned to committed history; validation must not re-derive
  historical classifications under newer rules.
