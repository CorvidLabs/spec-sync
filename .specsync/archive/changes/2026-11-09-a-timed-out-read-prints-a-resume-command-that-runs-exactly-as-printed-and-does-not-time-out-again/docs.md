---
change: a-timed-out-read-prints-a-resume-command-that-runs-exactly-as-printed-and-does-not-time-out-again
artifact: docs
---

# Docs

- `site/src/content/docs/cli.md` "Read timeouts": the resume command is shell-quoted and clears `SPECSYNC_TIMEOUT` when it was set.
- `CHANGELOG.md` Unreleased, Fixed.
