---
change: authenticate-finalized-policy-migrations-during-active-only-audit-without-weakening-policy-change-coverage
artifact: context
---

# Context

Algonest #48 is a same-PR finalized policy adoption, head72465f1. Official SpecSync6.1.1 audit rejects it because check_project_with_command_output checks only active records for policy coverage. The immutable package explicitly covers .specsync/sdd.json, but records is empty after finalization. Algonest #49's audit passes against #48's stacked base; this does not repair #48 versus main. The separate missing-Git-object error on #49 was resolved by fetching authentic history and is not part of this source repair.

Read specs/change/change.3md: active audits must remain focused on living records, archives must be authenticated selectively, and a same-PR archive must retain its exact delivery obligations. Do not restore or edit old approvals or workflow baseline. Current release is6.1.1; this work proposes an upstream PR, not a release or merge.
