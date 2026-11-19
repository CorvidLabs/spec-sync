---
kind: evidence
id: 2026-11-19-correction-to-the-round-3-entry-recorded-at-985de940-it-says
aliases: []
claim: "Correction to the round-3 entry recorded at 985de940: it says the (c), (d) and (e) tests fail on the old code. They were not run on 0d31218a, and they cannot compile there because they use main_commit and MAX_DELTA_REBASES. Their failing inputs are Codex's own reproductions on 0d31218a. Red-first was run for F1 and F2 (both fail on the code before them) and for the #654 squash-merge test (fails without its fix). The tester's regression is red by the tester's run on 0d31218a."
result: not-run
evidence_kind: product-test
revision: 44f37fb911805e7951b937ef9c74c8952ebf40dd
content: 34671f2b2e667b79f12921b2e20a45a6f46aa96526e9487bd79b46e7473e593d
environment: local macOS 26, rust 1.89.0, branch 1085-rebase-stale-modified-delta at 44f37fb9
criteria: [CHANGE-6.a]
requirements: [REQ-change-161]
contracts: []
limitations: []
observed: 2026-11-19T06:04:40Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Correction to the round-3 entry recorded at 985de940: it says the (c), (d) and (e) tests fail on the old code. They were not run on 0d31218a, and they cannot compile there because they use main_commit and MAX_DELTA_REBASES. Their failing inputs are Codex's own reproductions on 0d31218a. Red-first was run for F1 and F2 (both fail on the code before them) and for the #654 squash-merge test (fails without its fix). The tester's regression is red by the tester's run on 0d31218a.**

Recorded as not-run in local macOS 26, rust 1.89.0, branch 1085-rebase-stale-modified-delta at 44f37fb9 at revision 44f37fb91180, observed 2026-11-19T06:04:40Z.
