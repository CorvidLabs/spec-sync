---
change: CHG-0003-finalize-specsync-5-0-release-consistency-and-parallel-validation
artifact: tasks
---

# Tasks

- [x] Make effective-contract scratch paths unique across concurrent validations.
- [x] Add a deterministic parallel allocation regression.
- [x] Correct canonical spec and companion contradictions found by the semantic audit.
- [x] Correct migration, workflow, agent, comparison, and README wording; update PR evidence after CI.
- [x] Account for unrelated `cmd_score`/`hooks` freshness warnings: strict API validation is green and neither source file changes in this PR.
- [x] Run focused, CI-style serial/parallel, all executable example, and full repository checks locally; packaged consumer remains independently gated in CI.
- [x] Clean generated build artifacts and obtain a fully green PR matrix.
