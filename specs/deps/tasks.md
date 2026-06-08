---
spec: deps.spec.md
---

## Done

- [x] `build_dep_graph` / `extract_module_from_dep_path` — build the module graph from spec frontmatter
- [x] `validate_deps` — report missing dependency targets (errors + `missing_deps`)
- [x] `detect_cycles` — DFS-coloring cycle detection with full chain reporting
- [x] `extract_imports` for Rust, TypeScript/JavaScript, and Python (regex-based)
- [x] `check_undeclared_imports` — warn on undeclared cross-module imports, never flag self-imports
- [x] `topological_sort` — deterministic DAG ordering, `None` on cycles
- [x] Skip cross-project refs during local validation
- [x] Inline unit tests for graph build, validation, cycles, import extraction, and topo sort
- [x] Populate requirements.md with user stories and acceptance criteria (2026-04-10)

## Open

- [ ] No end-to-end integration fixture asserting `specsync deps` CLI output for missing/circular deps (only inline unit tests exist)
- [ ] Import extraction is regex-based and may miss aliased/multiline import forms

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
