---
kind: evidence
id: 2026-11-20-1134-round-one-corvid-agent-s-review-codex-s-two-p2s-red-first
aliases: []
claim: "#1134 round one (corvid-agent's review, Codex's two P2s). Red first b7c738aa: with nothing canonical at the reference (specs_dir outside the project, or a configuration that does not parse) the diff ran with no pathspec and read every requirements.md (two tests, regression from #1108's head); a registered spec's directory below an unreadable directory was passed by before the scan named it. Four more tests cover criteria that had none (REQ-change-175 c3, 176 c2, 177 c2, REQ-cmd-change-045 c2). Fix aad978c4: return before the diff when the reference scope is empty; the scope-directory check names a path or metadata it cannot read; testing.md maps each criterion to its test. Lane at 30c27ad4 (3137 unit, 660 integration, 62/62, 100%); audit --strict 0; real repos as round one (starling archived tip exit 0 no blocker; active commit, peck, spec-sync and chough mains the same lines as #1108's build)."
result: passed
evidence_kind: product-test
revision: 30c27ad45f2771652f3fea58b94471557b24ca91
content: a15ddfbd4fac7bb335c6370b8a45001fb2b8c863ca4ff7da20579e1fd35061c3
environment: local macOS 26, rust 1.89.0, branch 1112-1113-1116-1117-lost-requirements at 30c27ad4
criteria: [CHANGE-17, CHANGE-17.a, CHANGE-17.c]
requirements: []
contracts: []
limitations: []
observed: 2026-11-20T17:02:40Z
actor:
  - agent: claude-code
    terminal: t81
---
**#1134 round one (corvid-agent's review, Codex's two P2s). Red first b7c738aa: with nothing canonical at the reference (specs_dir outside the project, or a configuration that does not parse) the diff ran with no pathspec and read every requirements.md (two tests, regression from #1108's head); a registered spec's directory below an unreadable directory was passed by before the scan named it. Four more tests cover criteria that had none (REQ-change-175 c3, 176 c2, 177 c2, REQ-cmd-change-045 c2). Fix aad978c4: return before the diff when the reference scope is empty; the scope-directory check names a path or metadata it cannot read; testing.md maps each criterion to its test. Lane at 30c27ad4 (3137 unit, 660 integration, 62/62, 100%); audit --strict 0; real repos as round one (starling archived tip exit 0 no blocker; active commit, peck, spec-sync and chough mains the same lines as #1108's build).**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1112-1113-1116-1117-lost-requirements at 30c27ad4 at revision 30c27ad45f27, observed 2026-11-20T17:02:40Z.
