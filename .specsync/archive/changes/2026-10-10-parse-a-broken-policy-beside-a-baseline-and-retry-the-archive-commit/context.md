---
change: parse-a-broken-policy-beside-a-baseline-and-retry-the-archive-commit
artifact: context
---

# Context

Review on pull request 807. An adopted project already has `.specsync/workflow-v2-baseline.json`, and `change new` returned workflow version 2 without parsing `.specsync/sdd.json`. A broken policy still opened a change.

`change finalize` moved the package and then committed. If the commit failed, the change was already archived, and running finalize again refused the state instead of committing the package. The next line still said only to merge.

`specsync rules` used `Path::join` for the principles file, so an absolute path or `..` could be reported as present. Approval rejects those paths.
