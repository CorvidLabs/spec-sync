---
kind: bug
id: 2026-11-19-repository-steered-writes-reads-and-caches
aliases: []
title: Repository-steered writes, reads and caches
status: open
classification: product-defect
scope: blocking
impact: Overwrite of files outside the clone or of a Git hook (code execution at the next commit), a check that passes failing specs, a hang or out-of-memory; 2.0.0 through 6.3.0 depending on the command
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements: []
cases: []
evidence: []
receipts: []
fixture: tests/integration/project_writes.rs, tests/integration/repository_inputs.rs and src/hash_cache.rs tests; each fixture a temporary directory, links only inside it
environment: local-macos-arm64
revision: 1b3974e9a025f37ce712d4ad627e99f780d6f207
found_by: claude-code (hc t5r) (manual report)
actor:
  - agent: claude-code
    terminal: t5r
related: []
created: 2026-11-19T20:10:43Z
updated: 2026-11-19T20:10:43Z
---
# Repository-steered writes, reads and caches

**What happened:** A cloned repository could steer writes outside itself or into .git through committed links (change.md, artifacts, the change workspace, --fix --backup, init-registry), hide a failing spec with a committed hash cache, send spec discovery over the disk with specs_dir, exhaust memory through changelog or import --from-dir, and a monorepo subproject's links that stay in the clone were refused

Actual: The outside file or a Git hook was overwritten or created, a forged pass was replayed, the walk was unbounded, memory was exhausted, and subproject links were refused as leaving the project

**Steps:** Trigger: change answer/depend/supersede/approve/start/new, check --fix --backup, init-registry, a plain check with a committed .specsync/hashes.json, deps/view/rehash with specs_dir or a specs link outside, changelog over a huge historical blob, import --from-dir with a link out, hooks install in a subproject

**Expected:** Nothing outside the Git work tree or inside .git is written; a tracked cache is not used; configured directories stay inside and every walk is bounded; blobs and imports are bounded; links that stay inside the repository work in a subproject

**Why it matters:** Overwrite of files outside the clone or of a Git hook (code execution at the next commit), a check that passes failing specs, a hang or out-of-memory; 2.0.0 through 6.3.0 depending on the command

**Fix:** Not fixed yet.

## Log

- 2026-11-19T20:10:43Z · Reported (manual report) · by claude-code (hc t5r)
