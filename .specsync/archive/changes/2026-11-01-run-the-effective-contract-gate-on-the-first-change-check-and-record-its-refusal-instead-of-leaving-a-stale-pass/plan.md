---
change: run-the-effective-contract-gate-on-the-first-change-check-and-record-its-refusal-instead-of-leaving-a-stale-pass
artifact: plan
---

# Plan

## Approach

Close the one run in which the lifecycle's effective-contract gate does not ask: the first
`change check` after materialization. When that gate refuses a module the change declares,
record the refusal as a failed verification attempt instead of returning before anything is
written. The scoped pass, its command string, `specsync check`, configured enforcement and the
gate's own rule all stay as they are.

## Out of scope

- Making `change check` honor configured `enforcement`. That is the wrong mechanism (see
  `research.md`).
- Changing the gate's warning promotion or its suppressions.
- Making status run the gate.
- Changing `change audit` or acceptance.
- Changing `change_requires_strict_validation`, its hard-coded SpecSync paths and modules, or
  `summary.strict_validation_required`.
- Correcting the docs claim that the classification "adds validators". That is a recommended
  follow-up.
- The status-approval defect (Chough BUG-0002), which is a separate package.
- Release, tag or merge.

## Steps

1. Write the discriminator tests in `testing.md` and confirm they fail on f240d37.
2. Add private per-module attribution to `EffectiveContractOutcome` without changing any
   existing caller's output.
3. In `verify_change_locked`:
   - give the gate a roster that treats the verifying change as verifying,
   - return non-attributable gate errors as today,
   - hold declared-module errors until after the tasks and evidence checks,
   - record them as the second command entry of the attempt.
4. Run the controls: clean change, `--strict`, path-only scope, `--no-spec-change`, another
   change's module, `enforcement` variants, and an accepted or archived change.
5. Update `site/src/content/docs/workflow.md`, the change companions and the lesson in
   `specs/change/change.3md`. Approval materializes the delta.
6. Run the regression lanes, then `fledge lanes run pre-push` before any push.

## Risks

- **A change that verified green on 6.1.1 can start failing its first check.** That happens when
  its declared modules carry warnings it never fixed. Those changes could not finalize anyway;
  the failure now arrives one step earlier, is named, and is recorded. The PR states it.
- **Attributing gate errors by module must not drop any error.** Every message the gate produces
  is either attributed to a declared module or returned as today. A test asserts that the union
  equals the gate's full error list.
- **Two active changes declaring one module.** If another change's pending delta breaks that
  module, the failure is recorded against this change too. Finalization would refuse it for the
  same reason, so the record is accurate, but the cause sits in another change. The error text
  names the gate's findings so the operator can see where.
- **Test file size.** `semantic_application_resolves_registry_backed_canonical_paths.rs` is at
  3,997 lines and may also receive tests from the status-approval package. Together they stay
  under 5,000 (REQ-change-109).

## Constraints consulted

- `specs/change/change.3md` and REQ-change-091: in-process verification that spawns nothing.
- REQ-change-023 and the F4 status fix: no evidence claims a `--strict` run that did not happen.
  Attempts are append-only, and a corrected retry can pass.
- REQ-change-063: the gate's promotion and suppressions are kept.
- REQ-change-092: `canonical_applied` means materialization ran, not that the gate ran.
- REQ-change-109: the line limit.
- HI: SHIP-1.a, SHIP-1.d, SHIP-3.a and CHANGE-7 are linked. No new HI criterion is proposed.
