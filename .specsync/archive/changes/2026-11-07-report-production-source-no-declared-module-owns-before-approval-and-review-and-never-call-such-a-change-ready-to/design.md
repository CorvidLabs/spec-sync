---
change: report-production-source-no-declared-module-owns-before-approval-and-review-and-never-call-such-a-change-ready-to
artifact: design
---

# Design

Two checks, both reusing finalize's own ownership resolution, so the answer can never disagree with finalize.

1. **Approve** (`validate_declared_path_ownership`): stop skipping a no-spec-change change with no declared specs. Each existing affected production-source path is resolved the same way, so with no declared module it is unowned and refused. The refusal keeps today's per-path hints (REQ-change-120). It gains one line for a draft: `specsync change answer <id> affected_specs <modules>`, where the modules are the declared ones plus every undeclared module that owns a refused path.
2. **Ship preflight and ship-status:** a read-only ownership preflight walks the delivery inputs finalize would sign (the same discovery and `record_covers_project_path` filter as the acceptance manifest) and calls `acceptance_input_owners` with `UnownedProductionSource::Reject`.
   - Every input that fails with the ownership error becomes one blocker.
   - The blocker names the path, the owning module (by the same hint as approve), and the recovery: `specsync change send-back <id>`, then `specsync change answer <id> affected_specs <modules>`.
   - `ready_to_finalize` is false, ship-status lists the blocker, and `ship` (including `--dry-run`) exits non-zero without mutating anything, as for open tasks (REQ-cmd-change-031).

Rejected:
- Expanding directory scopes at approve: a directory covers files the change may never touch, so it would refuse changes finalize accepts.
- Keeping the check only at finalize: that is the bug.
