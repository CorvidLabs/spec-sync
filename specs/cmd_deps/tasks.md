---
spec: cmd_deps.spec.md
---

## Tasks

- [ ] Add a CLI test asserting `--mermaid`/`--dot` output shape (the renderers are private to the wrapper and have no direct test today)
- [ ] Add a CLI test asserting the non-zero exit when `report.errors` is non-empty

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented
- [x] Verified format dispatch (Json / Markdown+Github / Text+Table+Csv) and the topological build-order line for the text path
- [x] Verified the empty-graph `depends_on` hint fires only in diagram mode
- [x] Confirmed the delegate is covered by `deps` inline tests (graph build, missing dep, cycle detection, undeclared imports per language, topological sort) and the `dependency_spec_not_found_errors` integration test

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
