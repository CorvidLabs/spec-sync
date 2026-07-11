---
spec: view.spec.md
---

## Tasks

(none open)

## Done

- [x] `sections_for_role` visibility map for dev / qa / product / agent
- [x] `valid_roles` static role list
- [x] `view_spec`: role validation, frontmatter parse, section filtering
- [x] Role-specific header `# {module} (view: {role})`
- [x] Agent role: emit `**Status:**` and `**Agent Policy:**` (default "not set (default: full-access)")
- [x] Product role: append companion `requirements.md` body (frontmatter stripped)
- [x] `split_sections` splitting body on `## ` headings
- [x] `strip_frontmatter` helper for the appended requirements body
- [x] Error paths: unknown role, unreadable file, unparseable frontmatter
- [x] Populate requirements.md with user stories and acceptance criteria (2026-04-10)

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
