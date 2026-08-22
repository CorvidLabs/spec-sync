---
change: the-release-lane-must-be-able-to-read-the-tag-that-triggered-it
artifact: testing
---

# Testing

The lane cannot be exercised without pushing a tag, so the evidence is the measurement that
identified the defect rather than a test that reproduces it:

```
GitHub API   type=tag  tagger=0xLeif  target=89886855  msg="spec-sync 6.0.0-rc.1"
tag created  2026-08-22T00:05:07Z
run started  2026-08-22T00:05:15Z   (triggered by that tag, 8s later)
run verdict  "RC tag 'v6.0.0-rc.1' must be an annotated tag, not a lightweight tag"
```

An annotated tag on the server, refused by the run it triggered. Not a race and not a bad tag.

`release.yml` parses as YAML, and `resolve`'s checkout now carries `fetch-tags: true`.

The real verification is a `workflow_dispatch` run with `dry_run: true` after this merges, which
is also how the six jobs past `resolve` get exercised for the first time without spending a tag
per attempt.
