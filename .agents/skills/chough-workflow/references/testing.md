# Focused testing

Test what the change affects, not every criterion the product has.

## Choose the checks

Run `chough cases select --requirement <REQ> --criterion <ID> --path <changed path>`
for the affected scope. Add `--track copy-localization` when visible text,
terminology, comprehension or locale behavior changed, and `--track design-qa` when
visuals or interactions changed. The output lists selected cases, omitted cases with
reasons, and the project-required lane if one is configured
(`chough pref set required-lane <lane> --source …`). Also run proportionate
regressions for nearby behavior and the project's mandatory gates. Explain any
meaningful omission and record test debt in the native `testing.md`.

| Track | When | Typical method |
|---|---|---|
| functional | Always | Automated unit, integration or UI tests; manual checks where needed |
| copy-localization | Visible text or locale behavior changed | Deterministic string assertions plus reviewer notes |
| design-qa | Visuals or interactions changed | Rendered comparison with the canonical prototype at the target viewports and states |

## Definitions are not observations

Definitions live in `.chough/cases/<module>.cases.toml` (profile `chough-testing/1`,
see [the template](../templates/cases.toml)). Each new case has a curated stable ID
`TC-<module>-<title-slug>`, with lowercase ASCII letters and digits in nonempty runs
separated by single hyphens. The complete ID stays within the module-aware 100-byte
bound, and duplicate IDs are refused globally. Exact legacy `TC-<module>-NNNN` cases
remain valid without renumbering or changing their definition digests. Each case
links to HI criteria and requirements, setup, steps, explicit expected outcomes,
method, environments and expected artifact types. Several cases can cover one
criterion. Never edit an expected outcome to match a failure; retire a case
(`status = "retired"`) instead of deleting or reusing its ID.

`chough cases check` validates every definition, including its HI and requirement
links. The digest of a case ignores its position and title, so reordering never
changes identity, while editing what it asserts makes earlier observations
`stale-case-definition`.

A requirement an approved native change adds is not in the companion until the
scoped `specsync change check <id>` materializes it. Before then `cases check`, `run`
and `record` accept a case linking it when the checkpoint records that change
(`chough checkpoint --native-change <id>`), the change passes the build gate's native
checks and its delta for the case file's module adds or modifies the ID; `cases
check` lists such links under `approved_delta_requirements`. Anything else is
refused with the reason. Running or recording cases is never a `tasks.md` checkbox:
`change check` refuses while any box is unchecked.

## Failing-first evidence in the test stage

The test stage links three pieces of evidence for each regression or new behavior:

1. The commit that adds the failing tests, before the fix. Record it with
   `chough checkpoint --completed "Failing tests: <commit>"`.
2. The failing run on that commit: `chough cases run <ID> --environment <env>` records
   a failed observation, and `chough bug add ... --observation <id>` (or
   `chough bug update <bug> --observation <id>`) links it, so the failure is a
   receipt rather than a blocker.
3. The passing run after the fix: rerun the same case, which records a passed
   observation later than the failure.

Leaving test for accept needs, for each case the checkpoint references, a latest
executed observation that passed or that a bug links. Because cases can link the
approved delta's requirements before materialization, all three steps run inside
Chough, in order, rather than after the fact.

## Execute honestly

- Automated: `chough cases run <ID> --environment <env>` runs the binding without a
  shell and records an automatic `OBS-<slug>-<24-lowercase-hex>` observation with
  the captured log at exactly `.chough/artifacts/<observation-id>.log`. An optional
  `--observation-id` accepts only the new OBS form. An old-style ID supplied for a
  new write is refused with a remedy to omit the ID for automatic allocation.
  `cargo-test` passes only when exactly one test ran and passed; a filter matching
  nothing is `not-run`. A validator refusing to load or run a definition is not a
  behavioral `not-run` observation: run the bound regression directly when failure
  first evidence is needed before the validator can accept its new case ID.
  `fledge-task` and `command` pass only with exit 0 and every `expect_output`
  fragment present, so each needs at least one fragment. `timeout_seconds` bounds a
  run from 1 to 3600 (default 600); a run that times out is recorded failed. The
  command exits 3 when the outcome is not passed.
- A binding may declare `env`, variables set on top of the inherited environment,
  for example `env = { NO_COLOR = "1" }` when a tool should print plain text. Chough
  sets none by default. `env` is committed with the case, so never put secrets in
  it. A `cargo-test` binding cannot set, in any ASCII case, names that choose what
  cargo builds or runs: `PATH`, `CARGO`, `CARGO_HOME`, `RUSTC`, `RUSTDOC`,
  `RUSTC_WRAPPER`, `RUSTC_WORKSPACE_WRAPPER`, `RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`,
  `RUSTDOCFLAGS`, `CARGO_ENCODED_RUSTDOCFLAGS`, `RUSTC_BOOTSTRAP`, or any name
  starting with `RUSTUP_`, `CARGO_BUILD_`, `CARGO_TARGET_` (`CARGO_TARGET_DIR` stays
  allowed), `CARGO_HOST_`, `CARGO_PROFILE_` or `CARGO_UNSTABLE_`. Observations name
  the declared variables, not their values, and always state that inherited
  variables were not recorded.
- `expect_output` and the `cargo-test` count read the output with complete terminal
  escape sequences (colours, hyperlinks) removed, while the artifact keeps the raw
  bytes. Write fragments as the visible text; a fragment containing ESC is refused.
- Manual or agent-assisted: `chough cases record <ID> --method manual --verdict 1=pass
  --verdict 2=fail … --summary "…" --artifact <screenshot>` gives a verdict for every
  expected outcome; any fail makes the observation failed. Like `cases run`, it
  allocates an OBS ID with a random 96-bit suffix when `--observation-id` is omitted
  and exits 3 when the outcome is not passed. Direct `chough evidence add [ID]` also
  allocates when ID is omitted and applies the same explicit new-form and old-style
  omit-ID rules. Stored legacy observations remain readable and linkable without
  migration.

A failing case stays failed even when every other case passes. Do not rerun until
it passes and report only the pass. Capture the failure with
[the bug procedure](bugs.md).

AI judgment is never the only oracle when an assertion or a reproducible procedure
can decide. Keep rendered, copy and human judgments as observations with method and
limits. Magpie can run real commands, but its generic mode judges exit codes and
records skipped steps as passes; bind a Magpie run only through `command` with an
`expect_output` assertion, and only where it is installed.

## Artifacts

Artifacts stay inside the project; there are no external artifact paths. Keep
screenshots and other large files small. One may be ignored by Git: the observation
keeps its fingerprint, and a clone without the file reads `stale-artifact`, never
passed. Do not use Git LFS for them while SpecSync `change check` refuses tracked LFS
files (CorvidLabs/spec-sync#789).

## Retention

Routine reruns are prunable; proof that something still references is not.
`chough evidence prune` previews eligible and protected observation sets with reasons
and bytes, and changes nothing. It protects what the checkpoint, a bug, an active
native change or the most recent archived one lists, non-routine records, the newest
3 automated runs and the newest failure per case and environment, validation failures
and boundary passes, and anything younger than 14 days. Apply only with `--apply` under
`chough consent grant retention --source "<their words>"`, or when a human set
`chough pref set retention auto --source …` (default `off`), which also prunes after
`cases run` and `validate run`. Removals are logged in `.chough/pruned.jsonl`. Never
delete evidence by hand, and never report a `pruned` or `unavailable` observation as
passed.
