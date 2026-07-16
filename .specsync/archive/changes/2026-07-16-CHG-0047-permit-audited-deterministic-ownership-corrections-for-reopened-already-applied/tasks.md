---
change: CHG-0047-permit-audited-deterministic-ownership-corrections-for-reopened-already-applied
artifact: tasks
---

# Tasks

- [x] Add backward-compatible acceptance-owner correction records
- [x] Implement exact canonical-owner validation and atomic correction transition
- [x] Expose `change correct-owner` through text and JSON CLI surfaces
- [x] Permit only validated corrections in reopened-definition compatibility
- [x] Include corrected owners in exact acceptance manifests without delta replay
- [x] Add unit, integration, tamper, portability, and legacy-serialization coverage
- [x] Update workflow documentation and canonical companions
- [x] Preserve the protected legacy baseline ledger in authority acceptance manifests
- [x] Run pre-acceptance format, lint, unit/integration tests, docs, audit, and release-build verification; confirm strict is blocked only by post-acceptance lifecycle refresh
