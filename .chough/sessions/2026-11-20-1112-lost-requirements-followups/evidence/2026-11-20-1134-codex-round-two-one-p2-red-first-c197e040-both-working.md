---
kind: evidence
id: 2026-11-20-1134-codex-round-two-one-p2-red-first-c197e040-both-working
aliases: []
claim: "#1134 Codex round two (one P2). Red first c197e040: both working-tree scans dropped an error the directory listing yields, or one from reading an entry's kind (entries.flatten()), so a requirement there read as lost; the test drives the scan's entry step with injected errors, since a failing filesystem cannot be made. Fix 16dfbbc0: one entry step, listed_entry, keeps the error with its path for the cannot-tell refusal. Lane at bd79255b (3138 unit, 660 integration, 62/62, 100%; the first run lost one unrelated test, archived_upstream::review_and_finalize_stop_when_the_upstream_archive_cannot_be_read, to a packed object under load; it passed 3 of 3 alone and the rerun was green); audit --strict 0; real repos as before."
result: passed
evidence_kind: product-test
revision: bd79255bf1cc1c65fc023777a5de2636930340a2
content: 202ef3b3202b6a0a53f697275cc1060c92a9fdeb7c76992325de45eb4aa47635
environment: local macOS 26, rust 1.89.0, branch 1112-1113-1116-1117-lost-requirements at bd79255b
criteria: [CHANGE-17, CHANGE-17.c]
requirements: []
contracts: []
limitations: []
observed: 2026-11-20T18:44:06Z
actor:
  - agent: claude-code
    terminal: t81
---
**#1134 Codex round two (one P2). Red first c197e040: both working-tree scans dropped an error the directory listing yields, or one from reading an entry's kind (entries.flatten()), so a requirement there read as lost; the test drives the scan's entry step with injected errors, since a failing filesystem cannot be made. Fix 16dfbbc0: one entry step, listed_entry, keeps the error with its path for the cannot-tell refusal. Lane at bd79255b (3138 unit, 660 integration, 62/62, 100%; the first run lost one unrelated test, archived_upstream::review_and_finalize_stop_when_the_upstream_archive_cannot_be_read, to a packed object under load; it passed 3 of 3 alone and the rerun was green); audit --strict 0; real repos as before.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1112-1113-1116-1117-lost-requirements at bd79255b at revision bd79255bf1cc, observed 2026-11-20T18:44:06Z.
