---
change: drop-the-windows-binary-from-the-6-0-release-matrix-and-stop-claiming-windows-support-while-keeping-every-windows
artifact: testing
---

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
