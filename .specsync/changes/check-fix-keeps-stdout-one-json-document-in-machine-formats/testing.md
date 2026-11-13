---
change: check-fix-keeps-stdout-one-json-document-in-machine-formats
artifact: testing
---

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
