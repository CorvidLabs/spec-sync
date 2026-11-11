---
change: after-approve-next-action-names-check-commit-and-review-and-ship-refuse-a-verification-commit-that-lacks-the-change-s
artifact: research
---

# Research

Reproduced on current main `ad8544e9` (specsync 6.2.0) in
`/tmp/ss535-repro`, a throwaway no-spec documentation change scoped to
`README.md`. After approve, show and status named
`specsync change check document-the-greeter` with no `--commit`. Handoff
was already `conditional` and said to commit the work. Following the
`Next:` line, bare `change check` exited 0; `change review` exited 0 and
bound the pass to the seed SHA; `change ship` exited 0, wrote
`chore(lifecycle): archive document-the-greeter`, and included the
README edit in that commit, while `verification.json` and `review.json`
still named the seed. `ship-status` then reported the review stale.
`specsync check` exited 0. The August hook refusal did not fire.

Existing contract already nearby: REQ-cmd-change-033 (ship-status names
`--commit`), REQ-change-093 / invariant 42 (uncommitted affected_paths
are a conditional handoff; uncommitted `.specsync/` evidence is not),
REQ-cli-args-010 (bare `change check` leaves `--commit` false), and
REQ-change-132 (status names open tasks before the next command, and
today pins that command without `--commit`).
