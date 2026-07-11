---
change: CHG-0003-finalize-specsync-5-0-release-consistency-and-parallel-validation
artifact: research
---

# Research

GitHub run `29141177119` failed `change::tests::unified_gate_validates_code_against_effective_delta` under the parallel test harness. The exact merge commit passes the focused test and all 1,717 tests with `--test-threads=1`, isolating the failure to shared parallel state. Current Spec Kit documentation includes resumable workflows, JSON state, and human gates; current OpenSpec documentation includes OPSX verification, synchronization, and bulk-archive conflict handling. The comparison must distinguish deterministic blocking enforcement from agentic/non-blocking verification without claiming those capabilities do not exist.
