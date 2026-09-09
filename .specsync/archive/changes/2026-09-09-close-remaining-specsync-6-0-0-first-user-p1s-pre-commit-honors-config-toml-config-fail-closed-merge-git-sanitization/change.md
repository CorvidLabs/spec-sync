---
id: close-remaining-specsync-6-0-0-first-user-p1s-pre-commit-honors-config-toml-config-fail-closed-merge-git-sanitization
state: archived
type: bug_fix
base_commit: 7df304a20edd77a5cc32396a1239b08577605077
---

# Close remaining SpecSync 6.0.0 first-user P1s: pre-commit honors config, TOML config fail-closed, merge git sanitization, and 5.x upgrade docs

## Intent

Close remaining SpecSync 6.0.0 first-user P1s: pre-commit honors config, TOML config fail-closed, merge git sanitization, and 5.x upgrade docs

## Affected Canonical Specs

- `hooks`
- `config`
- `merge`
- `git_utils`
- `change`

## Acceptance Criteria

- init then add-spec then check then hooks install then git commit succeeds without --no-verify; malformed TOML (garbage, empty file, directory-as-config, invalid enforcement enum) makes rules/rehash/compact/deps/archive-tasks/view exit 1 with could not be loaded, same as malformed JSON; no-config-file still uses defaults; unmerged_paths spawns git through git_cmd so GITHUB_TOKEN/AWS_SECRET_ACCESS_KEY/SPECSYNC_TEST_SECRET are absent from the child; v1 Verifying next_action names verify then accept then archive; uncovered-path remediation uses --kind bug-fix; MIGRATION.md has a copy-pasteable v1 close-out with merge-then-archive, commit of workflow-v2-baseline.json and adoption-report.json, names both change check and that verify no longer runs verification_commands, and states adopt is silent on committed still-active v1; confidence-report rows for #653 and MCP sanitization match the tree

## No-spec Rationale

Not applicable
