# Lesson bundle: check-fix-keeps-stdout-one-json-document-in-machine-formats

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Check --fix keeps stdout one JSON document in machine formats
- **Kind**: BugFix
- **Specs**: cmd_check
- **Paths**: src/commands/check.rs, tests/integration/fix.rs
- **Acceptance**: check --fix --json and check --strict --force --fix --dry-run --json print exactly one JSON document on stdout, and each per-spec added or would-add line on stderr; text output is unchanged

## Evidence

- Verification commit: `a91b111ec2939e1d0897f277fb973ad21106e2c0`
- Base commit: `18385ef5ead84f0b51d70a29a6fed8a3e5ca222a`
- Verified by: `specsync check --spec cmd_check`

## From the change's context.md

# Context

#475 (filed against 5.2.0). Reproduction 1, `init --json`, was confirmed fixed on 2026-11-10.
Reproduction 2 still held: `auto_fix_specs` (`src/commands/check.rs`) printed each spec's
`would add N export(s)` (dry run) or `added N export(s)` line on stdout in every format, so
`check --fix --json` and `check --strict --force --fix --dry-run --json` put human lines before
the JSON object. The caller already gates its own summary and backup lines on text mode, and the
existing `fix_with_json_output` test searched stdout for the first `{` to work around it.

cmd_check's acceptance criteria say JSON output is a single object. The issue asks for exactly
one parseable JSON document on stdout, with progress kept on stderr if it is kept. The fix keeps
the per-spec lines on stdout for text and prints them on stderr for every other format, the same
stream `--fix` failures already use in every format. No requirement text changes.

Chough: spec-sync's `.chough/` is still the 0.5.x layout and the installed Chough 0.6.0 refuses
it; migrating it is its own PR and needs Leif's consent (raised with t370). This change runs the
SpecSync lifecycle only, as #1059 and #1062 did.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| cmd_check: JSON output is a single object | `fix::fix_with_json_output` (tightened to parse all of stdout) and `fix::fix_dry_run_with_json_output_is_one_json_document`, red before the fix and green after |
| cmd_check: `--dry-run` previews `--fix` without writing | the dry-run test asserts the spec bytes are unchanged |

## How it runs

`change check` runs only the in-process spec-code sync. The tests run locally with
`cargo test --test integration fix::` and `cargo test --bin specsync commands::check::tests::`, and
in hosted CI's full suite at the archived head.

## Where these lessons go

- `specs/cmd_check/cmd_check.3md`
