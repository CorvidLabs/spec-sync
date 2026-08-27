---
change: drop-the-windows-binary-from-the-6-0-release-matrix-and-stop-claiming-windows-support-while-keeping-every-windows
artifact: context
---

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
