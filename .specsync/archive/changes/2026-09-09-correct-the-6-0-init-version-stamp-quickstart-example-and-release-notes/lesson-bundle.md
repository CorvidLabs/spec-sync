# Lesson bundle — correct-the-6-0-init-version-stamp-quickstart-example-and-release-notes

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Correct the 6.0 init version stamp, quickstart example, and release notes
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change.rs, tests/integration.rs, tests/integration/quickstart.rs, examples/quickstart/, specs/cmd_init/requirements.md, specs/cmd_init/testing.md, CHANGELOG.md, README.md, site/src/content/docs/quickstart.md, site/src/content/docs/deltas.md
- **Acceptance**: A fresh specsync init writes 6.0.0 to .specsync/version; examples/quickstart passes specsync check --strict --require-coverage 100 from its own root and reports an added undocumented export; the [6.0.0] release notes no longer claim check prints an active-change count or that a post-merge binding job exists

## Evidence

- Verification commit: `6c81a6d6002abcd0b1c75e47604a85b7287f905e`
- Base commit: `d0fb1621577ee3fc7f7d1a4e39f55ebaa6ded7fe`
- Verified by: `specsync check --spec change`

## From the change's context.md

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

## From the change's design.md

# Design

Smallest change that makes each surface true, with nothing that alters runtime behaviour for an
existing project:

- The stamp is a one-token constant change. Existing `.specsync/version` files are never
  rewritten by 6.0, so no repository observes a difference until it runs `init` fresh.
- The quickstart spec keeps its prose and gains the canonical Public API table shape so the
  export validator can match `greet`; `status: active` is what the binary itself recommends in
  its draft notice. Paths are relative to the example root because that is where the README
  runs it.
- The example is pinned by integration tests rather than a CI step so it is exercised by every
  `cargo test`, including the release-candidate qualification lane.
- Release notes are corrected in place with a parenthetical naming what was wrong, in keeping
  with how this changelog records its own errata, rather than silently rewritten.

## From the change's testing.md

# Testing

## Automated

- `tests/integration/quickstart.rs` (new, registered in `tests/integration.rs`):
  - `quickstart_example_passes_strict_check_with_full_coverage_from_its_own_root` copies
    `examples/quickstart/` to a temp dir and asserts `check --strict --require-coverage 100`
    succeeds, prints `1/1 exports documented`, `0 warning(s), 0 failed`, and
    `File coverage: 1/1 (100%)`.
  - `quickstart_example_reports_the_readme_undocumented_export` appends the README's `farewell`
    function and asserts plain `check` succeeds while printing `Undocumented export 'farewell'`,
    and `check --strict` fails on it.
  - Both tests fail against the previous spec (0/1 coverage, export validation skipped).
- `commands::init::tests::write_current_layout_creates_full_structure` compares the stamp to
  `PROJECT_VERSION = SDD_VERSION`, so it pins `6.0.0` without a text change.
- `every_integration_test_file_is_registered` guards the new module registration.

## Manual (release binary, this review)

- Fresh `init` in an empty git repository → `.specsync/version` reads `6.0.0`; `check`,
  `change adopt`, `change audit`, `generate` exit 0.
- `examples/quickstart`: `check` from its own root shows the output now printed in its README;
  adding `farewell` yields the warning; `--strict` exits 1.
- Full suite before the change: 2465 unit + 410 integration green (two unit tests need a
  non-root user, two integration tests need `USER` set; both hold in CI).

## Where these lessons go

- `specs/change/context.md`
