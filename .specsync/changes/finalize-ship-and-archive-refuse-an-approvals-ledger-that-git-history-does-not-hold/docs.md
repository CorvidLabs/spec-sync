---
change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
artifact: docs
---

# Docs

- `site/src/content/docs/workflow.md`: before finalize, commit `approvals.json` with the change; what the
  refusal says and the way out for each case.
- `site/src/content/docs/cli.md`: the new refusal under `change finalize`, `change ship` and
  `change archive`, and the ship-status blocker.
- `CHANGELOG.md`, Unreleased, Changed: finalize, ship and archive refuse an approvals ledger history does
  not hold; a note that every machine running change commands on a repository should be on 6.3.0 or
  newer, because older versions rewrote committed records.
- `specs/change/change.spec.md`: the `files:` entry for the new test file. The check is crate-private,
  so no Public API row.
