## ADDED

### SPEC SECTION Internal Organization

The public `cmd_change` entry point remains a thin dispatcher. Private modules group rendering, ship-status projections, archive guidance, shipping and pull-request orchestration, repository inspection, owner-input resolution and lifecycle commits. Existing tests are organized by responsibility with shared fixtures. Module boundaries preserve every existing public API, complete function body and literal, snapshot-bound mutation output, read-scope lifetime, domain readiness predicate, NUL-aware literal lifecycle staging, ledger flooring, gate-before-push ordering and no-merge behavior; the split introduces no new public behavior.
