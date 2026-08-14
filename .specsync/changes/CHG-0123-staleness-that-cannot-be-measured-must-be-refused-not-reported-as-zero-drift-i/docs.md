---
change: CHG-0123-staleness-that-cannot-be-measured-must-be-refused-not-reported-as-zero-drift-i
artifact: docs
---

# Docs

CHANGELOG under Unreleased → Fixed, written from the consumer's side: a
dashboard reading `report --json` was told `"stale": false, "commits_behind": 0`
for a project whose staleness could not be determined, and a `--min-score` gate
could pass on freshness points awarded for a question git was never asked.
