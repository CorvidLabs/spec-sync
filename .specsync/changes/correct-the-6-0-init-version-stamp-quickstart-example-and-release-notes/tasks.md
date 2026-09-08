---
change: correct-the-6-0-init-version-stamp-quickstart-example-and-release-notes
artifact: tasks
---

# Tasks

- [x] Set `SDD_VERSION` to `6.0.0` and align `specs/cmd_init/requirements.md` and `testing.md`.
- [x] Map `examples/quickstart` to `src/lib.rs` as `status: active` with a Public API table.
- [x] Replace the quickstart README's expected output and "make it fail" text with what the
      binary prints; fix its dead README anchor.
- [x] Add `tests/integration/quickstart.rs` and register it.
- [x] Correct the two stale `[6.0.0]` release-note entries and record the fixes under
      `[Unreleased] › Fixed`.
- [x] Replace retired `CHG-NNNN` identities in `site/src/content/docs/deltas.md` and the README
      tree diagram; fix `quickstart.md`'s claim that `init` installs agents.
- [ ] Human: approve the definition (`change approve <id> --actor <name>`), run
      `change check <id> --commit`, record review, and `ship` before merge.
