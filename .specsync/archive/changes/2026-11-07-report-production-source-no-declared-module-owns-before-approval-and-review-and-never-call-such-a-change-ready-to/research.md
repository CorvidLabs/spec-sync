---
change: report-production-source-no-declared-module-owns-before-approval-and-review-and-never-call-such-a-change-ready-to
artifact: research
---

# Research

- `validate_declared_path_ownership` (src/change/sdd_version.rs) returns `Ok` for `no_spec_change` with empty `affected_specs`, and skips paths that do not exist yet.
- Finalize: `acceptance_manifest_internal` (src/change/collect_requirement_ids.rs) discovers delivery inputs and calls `acceptance_input_owners(.., UnownedProductionSource::Reject)`. That call errors "production source without deterministic canonical ownership" (src/change/consider_accepted_evidence_anchor.rs).
- Owners come from `affected_specs` (the spec's `files:` and configured `owns`) and from `acceptance_owner_corrections`; `@exact:` classes cover tests, fixtures and delivery metadata.
- Repro: a directory-scoped no-spec-change change over `src/billing/charge.ts` (owned by `billing`) approves, checks and reviews. `ship-status` says ready, then `ship` refuses with the #956 error.
