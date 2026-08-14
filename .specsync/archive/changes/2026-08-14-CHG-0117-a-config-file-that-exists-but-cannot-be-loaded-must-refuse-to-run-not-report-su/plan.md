---
change: CHG-0117-a-config-file-that-exists-but-cannot-be-loaded-must-refuse-to-run-not-report-su
artifact: plan
---

# Plan

Implementation and the full suite ran **before** `change new`, per #542: delivery scope
freezes at the interview, and blast radius only becomes visible at compile and test time.

## Sequence

1. Add `load_error` to `SpecSyncConfig` — runtime-only, not part of the file schema.
2. Set it at the two unreadable-file fallbacks in `config.rs`.
3. Set it at the parse-failure fallbacks in `validator.rs` — the site the repro uses.
4. Refuse in `load_and_discover`, the shared entry point.
5. Verify the broken-config repro exits 1, and that **both** controls are unchanged.
6. `fmt`, `clippy -D warnings`, full suite.

## A step that was reverted

Step 4 was first written into `compute_exit_code` and `exit_with_status`. Both take the
coverage report but not the configuration, so it required threading a new parameter through
**36 call sites across 8 files**. Reverted in favour of the choke point: fewer edits, and it
cannot be forgotten by a command that omits an argument.

## Rollout

A project with a malformed config fails where it previously passed — intended, since it was
passing against rules it had not loaded. A project with a valid config, or none at all, sees
no change.

The likely first encounter is someone who has been running with a broken config for a while
and has never seen the stderr warning. The message names the file and both ways forward, so
the remedy is available without reading the source.
