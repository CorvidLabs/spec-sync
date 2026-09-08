# SpecSync 6.0 pre-release review

Date: 2026-09-08. Tree: `main` at `d0fb162` plus the change package
`correct-the-6-0-init-version-stamp-quickstart-example-and-release-notes`.

The brief was narrow and specific: **is 6.0 safe to ship as a major that can stay a major** —
no contract that will force a 7.0 to correct, no breaking change hidden behind a minor. This
document records what was checked, how, what was found, and what a second reviewer should verify
independently rather than take from here. Every claim below was produced by running the release
binary or the suite in this tree, not recalled.

## Verdict

**Ready to tag once PR #762 lands, with one operational step before publishing.** The product
binary passed every gate this review could run. The two things that would stop a stable 6.0.0
from reaching consumers correctly are both outside the binary and both already identified by the
maintainer:

1. `release.yml` on `main` still requires exactly **three** platform evidence records
   (`authorize-release` and `release` jobs) while the qualify matrix produces **two** (Ubuntu,
   macOS). A correctly qualified candidate cannot be promoted or published. PR #762 changes both
   guards to two and pins them with a test against `REQUIRED_PLATFORMS`.
2. Publishing `v6.0.0` as a full (non-pre) release repoints `releases/latest`. Every consumer of
   the `@v4` / `@v4.5.0` action wrapper that passes no explicit `version:` resolves `latest` and
   will be moved onto 6.0 — and 6.0 changes the default enforcement mode from `warn` to `strict`
   (#647). Pin those consumers first, or accept that they go red on the next run.

Confidence that the 6.0 **contracts** are settled enough not to force a 7.0: **about 90–95%.**
The residual is listed under "What could still force a 7.0", and none of it is a defect this
review could fix; each is a design boundary the maintainer chose knowingly and documented.

## What was run

| Check | Result |
|---|---|
| `cargo build --release` on 1.89.0 (pinned) | ok |
| `cargo fmt --check`, `cargo clippy --release -D warnings` | ok |
| Unit suite (`cargo test --release --bin specsync`) | 2465 passed. Two tests (`non_git_walk_skips_volatile_trees…`, `all_files_reports_unreadable_specs_as_manual`) fail only when run as root, because root ignores `chmod 000`; both pass as an unprivileged user. |
| Integration suite (`cargo test --release --test integration`) | 408 passed, 6 ignored. Two `comment::*` tests fail only when `$USER` is unset (they run `change approve` without `--actor`); both pass with `USER` set, as it is on every CI runner. |
| New `tests/integration/quickstart.rs` | see "Fixed here" |
| `specsync check --strict --require-coverage 100 --force` on this repository | 62 specs, 0 warnings, 106/106 files, 0.34 s |
| `specsync change audit --strict` on this repository's 225 archived changes (v1 and v2 records) | passes on `main`; on this branch reports the uncovered paths until the change package is approved (expected) |
| `examples/sdd-lifecycle`, `sdd-concurrent-changes`, `sdd-five-epics` with the release binary | all three pass |
| `validate-release-version.py`, `validate-workflow-runtime-pins.py`, `test-validate-release-candidate.py` (50 tests) | all pass |
| `cargo package --list` | 114 files; `tests/` deliberately excluded by `include` |
| `cargo check --target x86_64-pc-windows-gnu` | ok (see "Targets") |
| `cargo check --target x86_64-unknown-linux-musl` | ok (see "Targets") |

### Fresh-project walk (release binary, empty git repo)

`init` → `check` → `change adopt --dry-run` → `change adopt` → `change audit` → `generate` →
`check`: every step exits 0, `init --json` is clean JSON, `check --json` is clean JSON, `adopt`
flips only `enabled`. MCP `initialize` / `tools/list` / `resources/list` answer with five tools
and four resources.

### Forward-compatibility probes (what a later 6.x could write, read by 6.0)

| Probe | 6.0 behaviour |
|---|---|
| Unknown key and unknown table in `.specsync/config.toml` | `Warning: unknown key "future_key" in config.toml (ignored)`; exit 0 |
| `sdd.json` with `"version": 3` and an unknown field | accepted silently; treated as a v2 policy |
| `state.json` with an unknown field, same `workflow_version` | read fine; **the field is dropped on the next rewrite** (`change answer`) |
| `state.json` with `workflow_version: 3` | refused with `was written by a newer SpecSync (workflow version 3); upgrade specsync to read it`; `list`, `status`, `audit` all exit 1 |
| `.specsync/version` containing `6.5.0` | ignored; nothing in 6.0 reads the value except `migrate`, which compares it to `4.0.0` |
| A v2 record with `workflow_version` and `workflow_origin_version` stripped (what a 5.2 writer does, #603), uncommitted and committed | refused: `workflow-v1 change … was not present at the trusted pre-v2 cutoff`; `status` and `audit` exit 1. The downgrade is detected, as `MIGRATION.md` (PR #761) claims |

The dropped-unknown-field row is the one to understand. It does **not** force a 7.0: a later
6.x that adds a field to `state.json` must already tolerate its absence, because every record
6.0 wrote lacks it; and when a field is load-bearing, bumping `workflow_version` locks 6.0
readers out with a clear message rather than corrupting anything. The maintainers removed
`deny_unknown_fields` from 17 evidence structs for exactly this reason (CHANGELOG, "Evidence
persisted to disk is deliberately tolerant"). Preserving unknown fields through a rewrite would
be nicer (#434) and is additive if wanted later.

## Fixed here

Three defects, none covered by PR #761 or #762:

- **`SDD_VERSION` was `"5.0.0"`**, so a 6.0 binary stamps every fresh `.specsync/version` with
  the 5.0 layout version. No behaviour depends on the value today; it is fixed because the stamp
  is the one place a later 6.x can distinguish a 6.0-initialized tree, and a wrong value cannot be
  recalled once committed downstream. Existing stamps are untouched.
- **`examples/quickstart/` could not validate from its own root.** Its spec mapped
  `examples/quickstart/src/lib.rs` (outer-repo relative) and was `status: draft`, so the README's
  "make it fail" loop could not fire and the "expected output" was a 4.x-era format. Nothing in CI
  ran it. Now mapped as `src/lib.rs`, `active`, with a Public API table; the README shows what the
  binary prints; two integration tests pin the three promises.
- **Two `[6.0.0]` release-note entries described removed behaviour**: `check` printing an
  active-change count, and a post-merge merge-binding job (deleted in #499).

Also: retired `CHG-NNNN` identities in `site/…/deltas.md` and the README tree diagram (missed by
#762), and `quickstart.md`'s claim that `init` installs agents (it does not; `agents install`
does).

## Findings not fixed here, with reasons

| Finding | Why not here |
|---|---|
| `release.yml` three-vs-two evidence guards | PR #762 (draft) fixes it with tests; duplicating it would conflict. **This is the release blocker.** |
| README workflow block still uses `CHG-0001-add-passkeys`; `cli.md` `supersede` example uses flags the binary rejects (`--module`, missing `--digest`) | PR #762 rewrites both. |
| `docs/ADOPTING.md` pins `v6.0.0-rc.2` and contradicts itself on squash cost | PR #761 rewrites it. |
| `change answer` (and every verb that validates the workflow-v2 baseline) fails in a **shallow clone** with `workflow-v2 baseline cutoff must be an ancestor of its introduction's first parent`; `change new` succeeds first | Environment limitation with an opaque message. `MIGRATION.md` already says `fetch-depth: 0`. Worth a message naming the shallow clone; not a contract. |
| `change list --json` / `change status --json` emit a bare array when healthy and an object (`changes`, `unreadable`) when degraded | Deliberate (5.x consumers parse the array; the degraded shape comes with a non-zero exit). Shape-varying JSON is the one 6.0 contract most likely to be regretted; see below. |
| RC binaries self-report `6.0.0`, not `6.0.0-rc.N` (#675) | Cosmetic for the RC channel; irrelevant after the stable tag. |
| `sdd.json` `version` stays `1` after `change adopt` on a 5.x project while the v2 baseline says otherwise (CHANGELOG L1381) | Recorded by the maintainer as two sources of truth; 6.0 reads both consistently. |

## What could still force a 7.0

Ordered by likelihood, none above "low":

1. **Top-level JSON arrays** on `change list` / `change status`. There is no room to add a
   sibling field to an array. If 6.x ever needs one, the choice is a new flag or a 7.0.
2. **`workflow_version` bumps as the only lock-out mechanism.** Correct, but every bump strands
   older 6.x readers; frequent bumps would feel like a 7.0 in slow motion. Keep additions
   default-tolerant and bump rarely.
3. **Windows.** Dropping the binary is documented as a 6.0 decision; the `#[cfg(windows)]` code is
   retained but nothing in CI compiles it. `cargo install specsync` on Windows is promised in the
   README. See "Targets" for this review's check.
4. **`--enforcement` default `strict`.** Already BREAKING in 6.0, with `--enforcement warn` and the
   config key as escapes; nothing further to change.
5. **The slug identity** refuses duplicates rather than suffixing them. Documented as deliberate
   (archive directories are `<date>-<id>`). A future need for same-description changes would be a
   new feature, not a break.

## Targets

`cargo check` type-checks the crate for a target without linking, which is exactly what the
retained `#[cfg(windows)]` code has never received in CI.

- `x86_64-pc-windows-gnu`: **passes** (with `gcc-mingw-w64-x86-64` for the tree-sitter grammars). The retained `#[cfg(windows)]` code type-checks on 1.89; the README's `cargo install specsync` promise for Windows holds at least to that point. A link step and a runtime were not exercised.
- `x86_64-unknown-linux-musl`: **passes** (with `musl-tools`), matching the published `linux-x86_64-musl` asset target.

## For a second reviewer

Do not trust this document for these; re-run them:

1. `cargo test --release` as a non-root user with `USER` set — expect 2465 + 410, 6 ignored.
2. `SPECSYNC_BIN=$PWD/target/release/specsync bash examples/quickstart/../sdd-lifecycle/run.sh`
   and `cargo test --release --test integration quickstart`.
3. The forward-compat table above, by hand, in a scratch repo — it takes two minutes and is the
   part that matters most for "no 7.0".
4. That PR #762's two `release.yml` guards match `REQUIRED_PLATFORMS` **after** it merges, and that
   the `[Unreleased]` compare link still satisfies `validate-release-version.py`.
5. That the three consumer repositories named in #647 carry an explicit `version:` before
   `v6.0.0` is published as a non-pre-release.
