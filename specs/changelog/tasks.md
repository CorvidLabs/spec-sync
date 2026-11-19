---
spec: changelog.spec.md
---

## Tasks

## Post-5.0 Test Debt

- [ ] Add a CLI-level integration test for `specsync changelog <range>` (the `generate_changelog` git path is unit-tested via `setup_git_repo`, but the subcommand wiring is not)

## Done

- [x] Populate requirements.md with user stories and acceptance criteria (2026-04-10)
- [x] Implement frontmatter diffing (incl. agent_policy/implements/tracks) and section diffing
- [x] Implement `parse_range` with `..` validation and three formatters (text/JSON/markdown)
- [x] Git-backed `generate_changelog` tests against a temp repo
- [x] Ask a blob's size before reading it and skip one over the spec bound (REQ-changelog-002). Evidence: `tests/integration/repository_inputs.rs`.

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
