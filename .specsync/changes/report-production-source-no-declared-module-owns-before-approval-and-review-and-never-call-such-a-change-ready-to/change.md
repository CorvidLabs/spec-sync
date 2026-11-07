---
id: report-production-source-no-declared-module-owns-before-approval-and-review-and-never-call-such-a-change-ready-to
state: implementing
type: bug_fix
base_commit: 9dd2affdca4628156835a6a0488e32a2f4702e45
---

# Report production source no declared module owns before approval and review, and never call such a change ready to finalize

## Intent

Report production source no declared module owns before approval and review, and never call such a change ready to finalize

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- change approve refuses a no-spec-change draft whose existing affected production-source path a module owns, naming the path, the owning spec and module, and specsync change answer <id> affected_specs <modules>; running that command lets the draft approve.
change ship-status and change ship --dry-run run the canonical-ownership check finalize runs over the delivery inputs: a production-source input no declared module owns is a blocker naming the path, the owning module, and the recovery (specsync change send-back <id>, then specsync change answer <id> affected_specs <modules>); neither says ready to finalize, and the dry run exits non-zero.
The integration test with a directory-scoped no-spec-change change runs that recovery and ships; it and the approve unit test fail on main 9dd2affd and pass with the fix.
Ownership is decided exactly as before (REQ-change-053): a path that does not exist yet is still enforced at finalize, an owned change ships unchanged, and no gate, bound or timeout is relaxed.

## No-spec Rationale

Not applicable
