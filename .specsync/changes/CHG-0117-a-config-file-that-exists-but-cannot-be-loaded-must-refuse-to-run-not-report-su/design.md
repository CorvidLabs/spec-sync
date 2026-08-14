---
change: CHG-0117-a-config-file-that-exists-but-cannot-be-loaded-must-refuse-to-run-not-report-su
artifact: design
---

# Design

## The condition travels with the configuration

`SpecSyncConfig` gains a runtime-only `load_error: Option<String>`, set wherever loading
falls back to the built-in defaults **because a file existed and could not be used**.

It lives on the config rather than being handled at the point of detection because detection
and consequence are far apart: the loader knows the file is broken, but only a command knows
whether it is about to report a verdict. Warning at the detection point and returning
defaults is exactly what produced #570.

Absence of a config file deliberately does **not** set it. The defaults are then the intended
configuration, not a substitution.

## Four fallback sites, two shapes

| site | condition |
|---|---|
| `config.rs` ×2 | file exists but could not be read (not UTF-8, permissions) |
| `validator.rs` ×2 | file read but failed to parse, during retained discovery |

The parse-failure site is the one #570 actually reproduces through, and it does not match the
textual shape of the other two. A fix pattern-matched on the first shape would have compiled,
passed review, and left the reported defect untouched — which is why the sites were
enumerated by behaviour rather than by grep.

## One choke point, not thirty-six

The refusal is applied in `load_and_discover`, the single function every spec-reading command
uses to obtain its configuration and spec list.

The alternative was to thread the condition into `compute_exit_code` and `exit_with_status`
alongside the coverage report. That is **36 call sites across 8 files**, and it fails open: a
command that forgets the new argument silently keeps the old behaviour. A choke point cannot
be forgotten, and it also covers commands that never compute an exit code — `generate` and
`scaffold` would otherwise scaffold into a `specs_dir` the project did not configure.

## Refusing rather than degrading

The refusal is a hard exit, not a warning escalated under `--strict`.

`--strict` means "treat warnings as errors", and this is not a warning: every rule the
project configured is absent. There is no severity at which "I ignored your configuration and
proceeded" is the right outcome, and `--strict` being opt-in would leave the default path —
the one CI most often uses — reporting success.

The message names the file and both ways forward: fix it, or remove it to accept the built-in
defaults deliberately. Deliberately is the operative word; the defect was that the choice was
made silently on the project's behalf.
