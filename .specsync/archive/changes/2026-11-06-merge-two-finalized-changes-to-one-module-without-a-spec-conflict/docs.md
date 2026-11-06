---
change: merge-two-finalized-changes-to-one-module-without-a-spec-conflict
artifact: docs
---

# Docs

Module specs for merge, cmd_merge, validator, change, cli, and cli_args record the driver and the duplicate-ID refusal. `hi/merge.md` holds MERGE-1 through MERGE-4 in Leif's words.

`specsync merge --install` is the setup step. It appends `merge=specsync` rules under the configured specs directory and sets the local driver to `specsync merge-file %O %A %B %P`.
