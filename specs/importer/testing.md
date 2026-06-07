---
spec: importer.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/importer.rs` inline tests | Unit | Validate Importer behavior close to implementation, especially `import_github_issue`, `import_jira_issue`, `import_confluence_page`, `render_spec`, `String`, `extract_requirements_pub`, `Vec<String>`, `slugify` |
| `tests/integration.rs` | Integration | Exercise Importer through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Importer contracts or source files.
- [ ] Run `fledge run test` and confirm Importer unit/integration coverage still passes.
- [ ] Review examples in `importer.spec.md` against observed behavior when touching src/importer.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| `JIRA_URL` not set | `import_jira_issue` returns `Err("JIRA_URL environment variable not set")` |
| `JIRA_TOKEN` not set | `import_jira_issue` returns `Err("JIRA_TOKEN environment variable not set")` |
| `CONFLUENCE_URL` not set | `import_confluence_page` returns `Err("CONFLUENCE_URL environment variable not set")` |
| `CONFLUENCE_TOKEN` not set | `import_confluence_page` returns `Err("CONFLUENCE_TOKEN environment variable not set")` |
