---
change: refuse-check-review-finalize-and-ship-while-a-change-has-an-open-task-naming-each-open-task-and-the-command-to-re-run
artifact: context
---

# Context

- Audit SS-M05 (2026-11-05): bypassing the incomplete-task guard left every selected test passing. The follow-up probe found the refusal gives no recovery and status repeats the check command. The 95% plan row S6 asks for a mutation-checked gate-level test and a named recovery.
- While scoping, a fixture showed a worse gap: a task reopened after a passing check was reviewed and finalized. Leif chose to widen this change to cover it ("Widen S6", 2026-11-05 20:24 MST, relayed by the orc t18). See #892.
- Rejected: hashing checkbox state into a digest. That would make ticking a task cost re-verification, or reapproval, contrary to CHANGE-6.c. A direct gate at each verb is enough.
- Human gates: scope approval and implementation review come from Leif through the orc; this agent never records them as Leif.
