# Lesson bundle — drop-the-windows-binary-from-the-6-0-release-matrix-and-stop-claiming-windows-support-while-keeping-every-windows

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Drop the Windows binary from the 6.0 release matrix and stop claiming Windows support, while keeping every Windows-content correctness guarantee
- **Kind**: Operations
- **Specs**: github, cli, cmd_migrate, change, commands, view
- **Paths**: .github/workflows/rc-assets.yml, .github/workflows/release.yml, .github/scripts/validate-release-candidate.py, .github/scripts/test-validate-release-candidate.py, action.yml, README.md, CHANGELOG.md, site/src/content/docs/integrations/github-action.md, site/src/content/docs/comparisons/adversarial-proof.md, site/src/content/docs/quickstart.md, src/commands/mod.rs
- **Acceptance**: The release and RC asset lanes build five artifacts, not six: no x86_64-pc-windows-msvc entry, no pwsh Compress-Archive packaging step, and no .zip glob remains in either lane. EXPECTED_ARTIFACT_ARCHIVES in validate-release-candidate.py no longer lists specsync-windows-x86_64.exe, so the exact-set artifact gate passes on the five archives actually produced instead of failing closed on a missing Windows directory. action.yml refuses a Windows runner with an actionable error naming WSL rather than 404ing on an asset that is no longer published. README, the site's Available Binaries table, the Multi-Platform Matrix example, the quickstart download note, and the adversarial-proof CI claim no longer state or imply that a Windows executable is shipped. Every correctness guarantee for Windows-authored content survives unchanged: parser CRLF normalization, strip_frontmatter, the .gitattributes eol=lf pins, is_reserved_module_name, validate_module_name, MAX_SLUG_BYTES, the portable_output_path and slash_normalized_relative_path helpers, and all cfg(windows) code and fixtures. Requirement wording that scoped those guarantees to 'every platform SpecSync ships a binary for' is rebound to the host platforms a repository may be checked out on, so narrowing the shipped set cannot narrow the guarantee. The Ubuntu/macOS/Windows release-candidate qualification lane is untouched.

## Evidence

- Verification commit: `7d7dcc138f94fb9b5490f4fe34025ed92577ce03`
- Base commit: `d508f144a1d965b395abfe45f23c8b4e8978cd5f`
- Verified by: `bash .github/scripts/test-classify-ci-paths.sh`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`, `cargo test cli::tests::`, `cargo test change::`, `cargo test commands::tests::`, `ruby --version`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`

## From the change's context.md

# Context

## The decision and the evidence behind it

Downloads of the v5.2.0 assets — the only stable release, published five weeks ago:

```
macos-aarch64   462
linux-x86_64    446
macos-x86_64    209
linux-musl        2
linux-aarch64     0
windows-x86_64    1
```

One Windows download in five weeks. Across every 6.0 RC the Windows figure sits at 0-2 with
a uniformity that reads as automated verification rather than people.

The second leg matters more than the first. Every job in `ci.yml` runs on `ubuntu-latest`;
the comment at `ci.yml:277` says so explicitly. The Windows executable we publish has never
been exercised by ordinary CI. That is not hypothetical: `rc.7` fixed a defect where
`specsync view` failed with "Cannot parse frontmatter" on *every spec in the project* in a
checkout with `core.autocrlf=true` — the whole `cmd_view` surface, broken on the platform we
shipped a binary for, for weeks, because nothing ran there. `specs/view/view.spec.md`
Invariant 8 records it.

6.0 is a major version. It is the release where dropping a platform is legitimate.

## The distinction this change is built around

Dropping the Windows **binary** is not the same as dropping Windows **correctness**, and the
two are easy to conflate because they use the same word.

A teammate on Windows commits CRLF files. A colleague on Linux reads them. A repository
authored on Windows carries CRLF frontmatter, backslash paths, and filenames that must not
collide with `CON` or `NUL`. None of that depends on SpecSync running on Windows — it
depends on SpecSync being correct about Windows-shaped *content*, and that bug class
outlives the platform entirely. All of it stays:

- `parser::parse_frontmatter` CRLF tolerance and `parser::strip_frontmatter`, the single
  canonical stripper (`specs/parser` Invariants 1, 13, 14).
- The `.gitattributes` `eol=lf` pins, which say in their own comment that they govern *this*
  repository's trees and are "not a substitute for readers that tolerate CRLF".
- `validate_module_name` / `is_reserved_module_name`, `MAX_SLUG_BYTES` and its `MAX_PATH`
  justification, `portable_output_path`, `slash_normalized_relative_path`.
- Every `#[cfg(windows)]` block, junction/reparse rejection, and Windows-shaped test fixture.

Nothing in this change deletes a line of it.

## Why the requirement wording had to move too

Three requirements phrase a Windows-content guarantee in terms of the shipped platform set:

- REQ-change-083: "a legal directory component **on every platform SpecSync ships a binary for**"
- REQ-change-084: "a name **a supported platform** reserves"
- REQ-commands-013: "a directory **some supported platform** cannot open"

Read literally, removing Windows from the shipped set removes Windows from the scope of all
three, and the reserved-name and `MAX_PATH` guards become unjustified by their own spec. The
guarantee was never actually about what we ship — it is about what a repository can be checked
out on. The wording is rebound accordingly. `specs/commands` Invariant 8 already had this right
("`validate_module_name` is platform-independent: every host rejects Windows-invalid
characters"), and REQ-commands-004 already says "on every host"; both are left untouched as the
model the others are moved toward.

## What is deliberately left alone

**The `qualify` lane stays on Windows.** `release.yml`'s `qualify` job runs the
`release-candidate` Fledge lane on `ubuntu-latest`, `macos-14` and `windows-latest`, and
`REQUIRED_PLATFORMS` in `validate-release-candidate.py` still names all three. It is kept for
three reasons:

1. It is the only place the `#[cfg(windows)]` code this change protects is compiled and run.
   Dropping it would leave that code unexecuted anywhere — the exact failure mode that
   produced the `view` defect above.
2. `docs/ci-confidence.md` lists "dropping Windows/macOS without an immutable-SHA
   release-candidate gate" as a named anti-pattern. The gate is that lane.
3. `AGENTS.md` records that the protected workflow runs it and that changing what it runs
   "requires a separately pinned required-workflow update" — out of band for this change.

It also leaves a coherent position rather than a contradictory one: SpecSync is still tested
on Windows; it is no longer *shipped* for Windows. Note that `cargo install specsync` on
Windows continues to work — what ends is the prebuilt executable, not the ability to build
from source.

**Historical records stay.** `CHANGELOG.md` entries for 1.0.0, 5.0.1 and 5.1.1 describe
Windows binaries that really were published; the archived change ledgers under
`.specsync/archive/` are digest-bound evidence. Both are history, and history is not edited —
the new `[Unreleased]` entry is the correct place to record the removal.

**`site/src/content/docs/mcp-security.md` stays.** Its Windows paragraphs are junction and
backslash confinement semantics (content correctness), plus one paragraph on `#[cfg(windows)]`
quarantine cleanup that documents code which still exists and is still compiled by the
qualification lane.

## From the change's design.md

# Design

No UI, layout, component, or design-token surface is touched. The change is release
plumbing, documentation prose, and requirement wording.

The one design-shaped decision is the shape of the failure a Windows user now meets. There
were two options:

- Leave `action.yml`'s Windows branch in place and let it fail on an HTTP 404 from a URL
  that no longer resolves.
- Refuse at OS detection with a message naming the platform and the supported alternative.

The second is chosen. A 404 on `specsync-windows-x86_64.exe.zip` tells the reader that a
download broke, which is the wrong diagnosis and sends them looking for a network or
permissions problem. An explicit `Windows is not a supported target as of SpecSync 6.0; run
SpecSync under WSL` tells them what actually changed and what to do about it, at the first
step rather than the third.

The documentation follows the same rule: state the position plainly, in the place a reader
is already looking (the binaries table, the install note), rather than as a footnote. Nothing
in the repository promises a future Windows binary, so nothing is phrased as a deprecation
awaiting restoration.

## From the change's testing.md

# Testing

## Automated

| Command | Covers |
|---|---|
| `python3 .github/scripts/test-validate-release-candidate.py` | `EXPECTED_ARTIFACT_ARCHIVES` and `REQUIRED_PLATFORMS`. Most tests build fixtures by iterating the two constants, so the five-artifact set is exercised end to end without editing them. Baseline before the change: 48 tests, OK. |
| `python3 -S .github/scripts/validate-workflow-runtime-pins.py` | Workflow runtime pins still resolve after the matrix edits. |
| `python3 -S .github/scripts/validate-release-version.py` | `action.yml` and `README.md` version surfaces still agree with `Cargo.toml` after both files are edited. |
| `cargo test` | The whole suite. The only `src/` edit is a doc comment, so this is a regression guard, not a target. |
| `cargo clippy -- -D warnings` | Not in `verification_commands`, so `change check` will not catch it. Run explicitly. |
| `specsync check --strict --require-coverage 100` | Spec/code coherence for the six touched specs. |

## The two failures this change has to not cause

Both are release-lane failures that would only surface at promotion time, long after merge.

1. **Exact-set artifact gate.** `require_exact_entries` fails on a missing *and* on an
   unexpected artifact directory. Removing the build matrix entry without removing the
   `EXPECTED_ARTIFACT_ARCHIVES` entry fails the release with "missing
   specsync-windows-x86_64.exe"; the reverse fails with "unexpected". Verified by running the
   validator test suite, which constructs its artifact fixture from that map.

2. **`fail_on_unmatched_files`.** `Create release` lists `artifacts/**/*.zip` and sets
   `fail_on_unmatched_files: true`. After this change no job produces a `.zip`, so leaving the
   glob turns every future release red. Checked by reading the step, not by a test — nothing
   in the repository executes `softprops/action-gh-release`.

Note the asymmetry in `upload-artifact`: `if-no-files-found: error` fires only when *no*
pattern matches, so the dead `.zip` globs in the build jobs would not have failed. They are
removed as dead configuration rather than as a fix.

## Manual verification

- `grep -rn "specsync-windows" .github/ action.yml README.md site/` returns nothing outside
  `CHANGELOG.md` and `.specsync/archive/`, which are history.
- The RC lane's header comment no longer says "six targets" / "six build jobs".
- `action.yml` on a Windows runner produces a readable error naming WSL, rather than a curl
  404 on an asset that no longer exists.

## Regression surface that must stay green — the point of the change

These are not new tests. They are the existing guarantees this change must leave untouched,
and the reason each exists is that it protects a Linux or macOS user reading Windows-authored
content:

| Guarantee | Where |
|---|---|
| CRLF frontmatter parses; body returns LF-only | `specs/parser` Invariants 1, 13; `src/parser.rs` |
| One canonical `strip_frontmatter` | `specs/parser` Invariant 14 |
| A CRLF-authored spec renders exactly as its LF twin | `specs/view` Invariant 8 |
| Windows device basenames rejected on every host | `REQ-commands-004`; `is_reserved_module_name` |
| Windows-invalid characters rejected on every host | `specs/commands` Invariant 8 |
| Slug length bounded by `MAX_PATH` | `REQ-change-083`; `MAX_SLUG_BYTES` |
| Unix literal backslashes stay data; only Windows normalizes | `specs/cmd_issues` Invariant 14 |
| Conflicted-file CRLF preserved | `specs/merge` Invariant 15 |
| Byte-exact CRLF preservation on compaction | `specs/compact` Invariant 9 |
| Junction / reparse-point escapes rejected | `specs/mcp` Invariants 18-25; `specs/manifest` |

`cargo test` covers all of them; none should change verdict. If any does, this change removed
something it was not allowed to remove.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-github-002 | Both build matrices now yield exactly the five artifacts `EXPECTED_ARTIFACT_ARCHIVES` names, cross-checked by parsing the workflows and the validator constant together; neither contains a Windows entry. `action.yml` no longer has a Windows download branch at all — a Windows runner exits at OS detection with a message naming WSL and `cargo install specsync`, so a pinned consumer cannot reach an asset that is not published. 48/48 release-candidate validator tests pass |
| REQ-change-083 | Delta rebinds the slug rule to "every platform a SpecSync repository may be checked out on, Windows included, whether or not SpecSync publishes a binary for that platform", and states the decoupling as its own acceptance criterion. `MAX_SLUG_BYTES` and its `MAX_PATH` 260 justification in `src/change.rs` are untouched, so the guarantee and the code that implements it still agree |
| REQ-change-084 | Delta replaces "a name a supported platform reserves" with "a name a host platform reserves, Windows device names included". `is_reserved_module_name` and its `RESERVED` list are unchanged, so refusal behaviour is identical before and after; only the scope sentence moved |
| REQ-commands-013 | Delta replaces "some supported platform cannot open" with "some host platform cannot open" and adds the explicit decoupling criterion. The `Public API` delta carries all 50 rows verbatim with one description cell changed, verified by row count before writing. `specs/commands` Invariant 8 ("platform-independent: every host rejects Windows-invalid characters") and REQ-commands-004 ("on every host") already had this right and are untouched |
| REQ-cli-010 | New requirement states the position the whole change turns on: the published binary set is Linux and macOS, and that set is not the set of content SpecSync must handle. `cargo test` passes with no CRLF, reserved-name, or path-separator behaviour changed; the only `src/` edit in this change is a doc comment |
| REQ-cmd-migrate-003 | New requirement keeps the no-symlink / no-Unix-specific-operations constraint and rebinds its justification to hosts a repository may be checked out on. `cmd_migrate` Invariant 6 (no symlinks, "fragile on Windows and confuse git") is unchanged |

## Where these lessons go

- `specs/github/context.md`
- `specs/cli/context.md`
- `specs/cmd_migrate/context.md`
- `specs/change/context.md`
- `specs/commands/context.md`
- `specs/view/context.md`
