# Lesson bundle: print-archive-restore-steps-that-git-and-specsync-read-literally-with-control-bytes-shown-as-text

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Print archive restore steps that Git and SpecSync read literally, with control bytes shown as text
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/historical_definition_digest_matches.rs, src/change_tests.rs, src/change_tests/literal_guidance.rs, src/change_tests/canonical_resolution_tolerates_inert_legacy_registry_stub.rs, src/change_tests/archive_post_move_failure_restores_exact_source_bytes_without_residue.rs, tests/integration/change.rs
- **Acceptance**: the printed corrupt-archive restore puts back only the named package when its path holds * or ?, and the printed archiving-commit lookup and the probes read it literally; every printed specsync change status command reads an id that begins with - as the id; a package path holding a newline or escape byte is printed with no control byte and, run through bash, still restores that exact directory

## Evidence

- Verification commit: `0ca562a384d597a493adf1df97f718a70b71c9c7`
- Base commit: `247b6d2a62b93c7f24cbafb56c93e754f9edf85c`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Three Codex P2s on #901 (the corrupt-archive restore step, CHANGE-7.a.2), deferred by Leif's call
and filed as #929, #930 and #931. All three are in `corrupt_archive_recovery` and the probes in
`corrupt_archive_signal` (`src/change/historical_definition_digest_matches.rs`). The coordinator
(t370) asked for one PR that closes all three, because they edit the same function.

- #929: `validate_change_id` accepts `*` and `?`, so a package path can hold them. Shell quoting
  stops the shell from expanding them, but Git still reads the path as a pathspec glob. The printed
  `git restore -- '<package>'` can then restore sibling archives and overwrite their uncommitted
  edits, and the `git diff --quiet` and `git log --diff-filter=A` probes are ambiguous the same way.
- #930: an id that begins with `-` loads, but the printed `specsync change status '<id>'` hands it
  to Clap as a flag, so the confirmation step reads no archive.
- #931: archive directories are found by the id in their `state.json`, so a directory name is
  never validated. A newline or ESC byte in it reached the printed `Next:` and `Handoff:` lines
  raw, inside the shell quotes.

#929 and #930 break REQ-change-093 as it stands ("cannot turn the copied step into a different
command"; "running the named step ... returns authenticated-history"). #931 has no requirement
yet, so it adds one acceptance line under the HI criterion CHANGE-7.b. Leif approved that sentence
in the orc tab on 2026-11-06 ("Capture both"); it sat on the unmerged g-literal branch and is
carried here unchanged. The red tests are also carried over from that branch.

Out of scope, and filed as a follow-up: other next actions that interpolate an id (for example
the plain `specsync change status <id>` resume line) do not put `--` before it either.

Chough: Leif's rule (relayed by t370 on 2026-11-13) is "make sure u use chough 0.6.0 for all
spec-sync changes". spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges, so red
tests and the fix are built now, and this change goes through a Chough 0.6.0 session after main is
merged in, before review.

## From the change's design.md

# Design

- Pathspecs: the printed restore and archiving-commit lookup, and both probes behind them
  (`archive_package_differs_from` and the `log --diff-filter=A` lookup), pass the package as
  `:(literal)<package>` (relative to the project root, which is where the step runs). The shell
  word is still one quoted word.
- Change id: the printed status commands become `specsync change status -- '<id>'` and
  `specsync change status --json -- '<id>'`, with options before `--`.
- Control bytes: a new `printable_shell_word` quotes a word that holds no control byte exactly as
  `shell_quote` does today (single quotes; `'` as `'\\''`). A word that holds one is written as
  `$'…'`, with `\\\\`, `\\'` and `\\xHH` for each control byte (C0 and DEL). So the printed
  step holds no control byte, and bash still reads the exact name. Only `corrupt_archive_recovery`
  switches to it; other callers of `shell_quote` are untouched.
- Nothing about the verdicts changes: the same three cases, in the same order, chosen by the
  same probes.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-093 (literal pathspecs, #929) | `the_printed_restore_puts_back_only_the_named_package_even_with_glob_characters`, `the_printed_archiving_commit_lookup_matches_only_the_named_package`, `the_package_probe_reads_the_package_as_one_literal_path` |
| REQ-change-093 (`--` before the id, #930) | `the_printed_status_command_reads_a_leading_hyphen_id_as_the_change_id` |
| REQ-change-093 (control bytes, #931; CHANGE-7.b) | `a_control_byte_in_an_archive_directory_name_is_printed_as_text_that_still_restores_it` |
| REQ-change-093 (unchanged verdicts) | the existing corrupt-archive handoff and recovery tests |

## How it runs

The tests in `src/change_tests/literal_guidance.rs` run what the step prints, through bash and
Git, or parse it with Clap. Locally: `cargo test --bin specsync literal_guidance` style filters
on each name, red before the fix and green after. Hosted CI runs the full suite at the archived
head.

## Where these lessons go

- `specs/change/change.3md`
