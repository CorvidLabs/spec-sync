---
kind: bug
id: 2026-11-19-specsync-reads-project-files-through-links-waits-on-fifos-and
aliases: []
title: SpecSync reads project files through links, waits on FIFOs and reads without a bound
status: verified
classification: product-defect
scope: blocking
impact: Whoever runs SpecSync, or Chough on top of it, in a cloned repository can have it hang or exhaust memory, or read a file outside the repository into its output
criteria: [SETUP-7]
requirements: []
cases: []
evidence:
  - 2026-11-19-bounded-project-reads/2026-11-19-on-base-3a02a5d5-with-only-the-new-tests-added-11-of-17
  - 2026-11-19-bounded-project-reads/2026-11-19-with-the-fix-all-17-project-read-integration-tests-and-the-3
receipts: []
fixture: "tests/integration/project_reads.rs: a temporary project beside an outside directory; FIFOs under a timeout and sparse files, never /dev/zero"
environment: local-macos-arm64
revision: 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef
found_by: claude-code (hc t5r) (a tester's private finding, handled with GHSA-7q5v-8jcc-rx54)
actor:
  - agent: claude-code
    terminal: t5r
related: []
created: 2026-11-19T14:30:43Z
updated: 2026-11-19T15:13:02Z
---
# SpecSync reads project files through links, waits on FIFOs and reads without a bound

**What happened:** A command waits forever, or reads until memory runs out, or reads a file outside the repository

Actual: Reads use std::fs::read and read_to_string on the project path, which follow any link, block on a FIFO and read without a bound; a link from .specsync/sdd.json to /dev/zero reached 227 GB on macOS and was OOM-killed on Linux

**Steps:** Trigger: Any SpecSync command that reads a project file, such as change status --json (which Chough's next and status run), in a repository where that file is a symbolic link to /dev/zero or outside the repository, a FIFO, or very large
1. Commit .specsync/sdd.json as a symbolic link to a FIFO or a device
2. Clone the repository and run specsync change status --json
3. The command never finishes

**Expected:** A link that stays inside the project is read; a link that leaves it, a dangling link, a non-regular file and a file over its bound are refused with an error naming the path and the reason, and the command never waits on the file

**Why it matters:** Whoever runs SpecSync, or Chough on top of it, in a cloned repository can have it hang or exhaust memory, or read a file outside the repository into its output

**Fix:** Verified: 2026-11-19-bounded-project-reads/2026-11-19-with-the-fix-all-17-project-read-integration-tests-and-the-3 passed at 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef.

## Log

- 2026-11-19T14:30:43Z · Reported (a tester's private finding, handled with GHSA-7q5v-8jcc-rx54); evidence: 2026-11-19-bounded-project-reads/2026-11-19-on-base-3a02a5d5-with-only-the-new-tests-added-11-of-17 · by claude-code (hc t5r)
- 2026-11-19T14:30:43Z · Added despite similar reports: This one is about reads; the two similar bugs are about writes · by claude-code (hc t5r)
- 2026-11-19T15:13:02Z · status: open -> verified; evidence: 2026-11-19-bounded-project-reads/2026-11-19-with-the-fix-all-17-project-read-integration-tests-and-the-3. Fixed by the native change never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound: every in-scope read goes through util's bounded project read, and a source scan keeps it that way. The tests that failed on 3a02a5d5 pass. · by claude-code (hc t5r)
