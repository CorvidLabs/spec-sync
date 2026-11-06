---
change: drop-the-ci-yml-path-filters-so-every-pull-request-starts-ci-and-classify-alone-picks-its-lane
artifact: context
---

# Context

PR #941 changed only `.chough/` records, to close the Chough sessions of merged changes. It never started CI. `ci.yml` starts only when a changed path matches a glob in its `pull_request` or `push` `paths:` list, and `.chough/**` was not in that list. So the required `Required CI gate` never reported, and the PR stayed BLOCKED.

A `workflow_dispatch` run on the branch went down the full lane and was green, but GitHub leaves dispatch suites out of a pull request's status rollup, so it did not count.

The same gap covers every tracked path the 27 globs miss:
- `scripts/`;
- the agent skill installs: `.agents/`, `.claude/`, `.codex/`, `.cursor/`, `.gemini/`;
- `.github/CODEOWNERS`, the issue templates and the PR template;
- `.gitignore`, `.gitattributes`, `LICENSE`, `.trust.toml`, `.attest.json`, `.augur.toml` and `.atlasignore`.

`classify-ci-paths.sh` already sends every path it does not name to the full lane, so the filters only ever decided whether CI started at all, never which lane it took.

Decisions, relayed by orc t18:
- Leif (2026-11-06, Herdr orc tab), "Capture it, fix CI": capture CI-6 in `hi/ci.md` as "A pull request that only changes Chough records still gets the checks the branch requires.", and fix CI in a separate small PR, failing test first.
- Leif, "Drop path filters" (option C of three): remove `paths:` from both triggers, so every PR starts CI and classify alone picks the lane.
  - Option A, rejected: add `.chough/**` and keep a list of known gaps.
  - Option B, rejected: add about 15 globs.
  - The test must show that `ci.yml` has no path filter, and that a `.chough`-only change and a `scripts`-only change each classify to a lane that reports the Required CI gate.
  - CI-6 as captured covers it.

Chough could not open a session for this change. A worktree off `main` refuses `chough session new` until #941 merges, and #941 is the PR this change unblocks. So evidence lives in SpecSync and the PR body.
