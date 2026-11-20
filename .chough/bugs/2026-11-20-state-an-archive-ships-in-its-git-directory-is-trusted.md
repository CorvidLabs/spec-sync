---
kind: bug
id: 2026-11-20-state-an-archive-ships-in-its-git-directory-is-trusted
aliases: []
title: State an archive ships in its git directory is trusted
status: open
classification: unclassified
scope: in-scope
impact: A false PASS from a plain check in an archive that ships .git or in CI with the cache inside the checkout; a user's .env deleted by a replayed journal; a link to a user's own file followed; folders created outside the clone. Private, with GHSA-7q5v-8jcc-rx54
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements: [REQ-util-006, REQ-change-162, REQ-hooks-009]
cases: []
evidence: []
receipts: []
environment: local-macos-arm64 (APFS), rustc 1.89.0
revision: 18d0716016784b15fc2e3cf1cd6138ac64463a98
found_by: claude-code (hc t5r) (manual report)
actor:
  - agent: claude-code
    terminal: t5r
related: []
created: 2026-11-20T03:58:02Z
updated: 2026-11-20T03:58:02Z
---
# State an archive ships in its git directory is trusted

**What happened:** A copy of a project that ships its .git replays the hash cache kept at .git/specsync/<constant key>; a journal there is replayed; a cache folder inside the project is trusted; a project folder named * counts as tracked by the enclosing work tree; a registry entry naming .env lets a journal delete it; a local core.hooksPath outside the work tree makes folders there

Actual: On e3d508ac with change 7's tests (3fcb6de7): all 3 new unit tests and 5 of 7 new integration tests fail; the read scan finds 10 new opens

**Steps:** Trigger: Run check, a lifecycle command or hooks install --precommit in an unpacked tarball of a working copy, in CI with HOME at the checkout, or in a project unpacked as * inside a home folder kept under Git

**Expected:** No state a repository or an archive of one ships is read as SpecSync's own; the confinement and the hooks folder are not widened by repository text

**Why it matters:** A false PASS from a plain check in an archive that ships .git or in CI with the cache inside the checkout; a user's .env deleted by a replayed journal; a link to a user's own file followed; folders created outside the clone. Private, with GHSA-7q5v-8jcc-rx54

**Fix:** Not fixed yet.

## Log

- 2026-11-20T03:58:02Z · Reported (manual report) · by claude-code (hc t5r)
- 2026-11-20T03:58:02Z · Added despite similar reports: Change 6's bug was state shipped in the work tree; this is the Git directory change 6 moved the state into, which an archive carries whole, plus the literal-pathspec, registry and hooks-scope gaps the re-check of change 6 found · by claude-code (hc t5r)
