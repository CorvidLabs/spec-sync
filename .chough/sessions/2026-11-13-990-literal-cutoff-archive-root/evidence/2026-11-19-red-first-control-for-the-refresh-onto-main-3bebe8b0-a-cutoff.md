---
kind: evidence
id: 2026-11-19-red-first-control-for-the-refresh-onto-main-3bebe8b0-a-cutoff
aliases: []
claim: "Red-first control for the refresh onto main 3bebe8b0: a_cutoff_without_an_archive_root_is_not_unreadable_for_a_pathspec_magic_project_directory fails, the defect #990 reports."
result: failed
evidence_kind: product-test
revision: 17e42ce125145d2d736413a627c0a37bf83bc6b6
content: ef97c85dde273b3b2a9bef64dc7fad8a4b9247e049e26b50cfb09f545d497ad7
environment: "red control: main 3bebe8b0 plus only the new tests"
criteria: [SPEED-1.a.2]
requirements: []
contracts: []
limitations: []
observed: 2026-11-19T07:18:35Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Red-first control for the refresh onto main 3bebe8b0: a_cutoff_without_an_archive_root_is_not_unreadable_for_a_pathspec_magic_project_directory fails, the defect #990 reports.**

Recorded as failed in red control: main 3bebe8b0 plus only the new tests at revision 17e42ce12514, observed 2026-11-19T07:18:35Z.

## Key failing lines

- a_cutoff_without_an_archive_root_is_not_unreadable_for_a_pathspec_magic_project_directory FAILED
