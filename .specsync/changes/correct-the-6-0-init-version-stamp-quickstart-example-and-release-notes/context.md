---
change: correct-the-6-0-init-version-stamp-quickstart-example-and-release-notes
artifact: context
---

# Context

Found during a pre-release review of SpecSync 6.0 whose brief was "nothing that forces a 7.0".
The review ran the release binary against fresh projects, the committed examples, and forward-
compatibility probes (unknown keys in `config.toml` and `sdd.json`, a `workflow_version` the
binary has never seen), and read the release notes against the tree.

Three defects survived the check, none of which any open pull request covers:

- `SDD_VERSION` in `src/change.rs` was still `"5.0.0"`, so a 6.0 binary stamps every fresh
  `.specsync/version` with the 5.0 layout version. Nothing in 6.0 reads the stamp's value
  (`migrate` compares it only to `4.0.0`), so behaviour is unchanged today — but a stamp is the
  one place a later 6.x can look to tell a 6.0-initialized tree from a 5.x one, and a wrong value
  cannot be corrected once committed downstream. This repository's own stamp (`5.0.1`) is left
  as it is; only fresh `init` output changes.
- `examples/quickstart/` — the example the README sends a new user to first — could not validate
  from its own root: its spec mapped `examples/quickstart/src/lib.rs` (outer-repo relative) and
  carried `status: draft`, so `check` reported the source as planned, coverage 0/1, and skipped
  export validation. The README's "make it fail" step therefore could not produce a warning, and
  the "expected output" block showed a format the binary has not printed since 4.x. Nothing in CI
  ran the example.
- Two `[6.0.0]` release-note entries described behaviour that later entries in the same release
  removed: `check` printing an active-change count, and a post-merge merge-binding job
  (`post-merge-archive.yml`, deleted in #499).

Constraints honoured: no canonical spec contract changes (the `SDD_VERSION` constant is already
in `change.spec.md`'s Public API table with a value-free description); the `cmd_init` edits touch
only its `requirements.md` and `testing.md` companions, one of which still said `init` offers a
first-change interview. PRs #761 and #762 own the README workflow block, `cli.md`, `workflow.md`,
`ADOPTING.md`, and the release-lane evidence count; this change deliberately stays off those
hunks except the README tree diagram line (`CHG-*/`) that #762 missed.
