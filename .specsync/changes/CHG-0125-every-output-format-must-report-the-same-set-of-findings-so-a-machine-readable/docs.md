---
change: CHG-0125-every-output-format-must-report-the-same-set-of-findings-so-a-machine-readable
artifact: docs
---

# Docs

CHANGELOG under Unreleased → Fixed, written for the consumer who was misled: a
CI job parsing `--format csv` saw zero rows on a failing tree, and an agent
reading `coverage --format json` saw no findings at all.

`specs/output/output.spec.md` documents the new public renderers and drops
`print_skipped_links`, now private.
