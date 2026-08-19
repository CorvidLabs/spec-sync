---
change: CHG-0152-a-populated-semantic-delta-must-not-report-as-empty
artifact: docs
---

# Docs

No site or `--help` change. Discoverability is the error at the moment of failure:

- a populated file with no `## Added`, `## Modified`, or `## Removed` heading names those
  values instead of saying the file is empty
- an invalid `##` heading names the same allowed values
- `### requirement` / `### spec section` are accepted, matching the already
  case-insensitive operation headings

`site/src/content/docs/deltas.md` already documents the uppercase grammar and is out of
scope (not in `affected_paths`).
