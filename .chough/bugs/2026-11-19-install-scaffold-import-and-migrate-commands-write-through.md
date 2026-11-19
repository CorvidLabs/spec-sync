---
kind: bug
id: 2026-11-19-install-scaffold-import-and-migrate-commands-write-through
aliases: []
title: Install, scaffold, import and migrate commands write through symbolic links
status: verified
classification: product-defect
scope: blocking
impact: Whoever runs one of these commands in a cloned repository can have a file of theirs changed or created, with their own permissions. Each needs the command to be run on purpose; none runs on a plain check
criteria: [SETUP-6]
requirements: []
cases: []
evidence:
  - 2026-11-19-symlink-sweep-writes/2026-11-19-on-base-7b6e4779-with-only-the-new-tests-added-15-of-19
  - 2026-11-19-symlink-sweep-writes/2026-11-19-with-the-fix-at-a33dcc6f-all-19-project-link-integration-tests
receipts: []
fixture: "tests/integration/project_links.rs: a temporary project beside an outside directory; every link points inside the fixture"
environment: local-macos-arm64
revision: 7b6e4779ba5b7d0fedd621c04a917fcb02e28c25
found_by: claude-code (hc t5r) (private symlink sweep after GHSA-7q5v-8jcc-rx54, and the builder's reading of sibling commands)
actor:
  - agent: claude-code
    terminal: t5r
related: []
created: 2026-11-19T05:05:09Z
updated: 2026-11-19T05:30:31Z
---
# Install, scaffold, import and migrate commands write through symbolic links

**What happened:** A file outside the repository is appended to, overwritten, rewritten or created

Actual: Each command builds its path with root.join and writes with fs::write, fs::copy or File::create, which follow the link. exists() guards are passed by a dangling link

**Steps:** Trigger: hooks install or uninstall, agents install, mcp lock --write, scaffold, add-spec, new, import, migrate or resolve --remote in a repository where the file the command writes, or a directory on the way to it, is a symbolic link, dangling or not
1. Commit CLAUDE.md as a symbolic link to a path outside the repository
2. Clone the repository and run specsync hooks install --claude
3. Read the file the link names: it holds the spec-sync block

**Expected:** A link that stays inside the project is written through; a link that leaves it, a dangling link and a non-regular target are refused with an error naming the path, and nothing outside the project changes

**Why it matters:** Whoever runs one of these commands in a cloned repository can have a file of theirs changed or created, with their own permissions. Each needs the command to be run on purpose; none runs on a plain check

**Fix:** Verified: 2026-11-19-symlink-sweep-writes/2026-11-19-with-the-fix-at-a33dcc6f-all-19-project-link-integration-tests passed at a33dcc6ffd6210e4998c376edab1d331083bcd7f.

## Log

- 2026-11-19T05:05:09Z · Reported (private symlink sweep after GHSA-7q5v-8jcc-rx54, and the builder's reading of sibling commands); evidence: 2026-11-19-symlink-sweep-writes/2026-11-19-on-base-7b6e4779-with-only-the-new-tests-added-15-of-19 · by claude-code (hc t5r)
- 2026-11-19T05:30:31Z · status: open -> verified; evidence: 2026-11-19-symlink-sweep-writes/2026-11-19-with-the-fix-at-a33dcc6f-all-19-project-link-integration-tests. Fixed by the native change never-write-a-project-file-through-a-symbolic-link: every write site goes through util's project file functions on a cap-std handle of the project root. The tests that failed on 7b6e4779 pass. · by claude-code (hc t5r)
