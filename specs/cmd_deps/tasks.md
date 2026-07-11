---
spec: cmd_deps.spec.md
---

## Tasks

- [x] Add CLI coverage for `deps --mermaid` output shape — Evidence: `deps_strict_mermaid_still_gates`.
- [x] Add a CLI test asserting the non-zero exit when `report.errors` is non-empty — Evidence: `deps_strict_gates_on_undeclared_imports` and strict diagram gating coverage.

## Post-5.0 Test Debt

- [ ] Add CLI coverage for `deps --dot` output shape.

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented
- [x] Verified format dispatch (Json / Markdown+Github / Text+Table+Csv) and the topological build-order line for the text path
- [x] Verified the empty-graph `depends_on` hint fires only in diagram mode
- [x] Confirmed the delegate is covered by `deps` inline tests (graph build, missing dep, cycle detection, undeclared imports per language, topological sort) and the `dependency_spec_not_found_errors` integration test

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
