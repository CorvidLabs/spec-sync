---
change: close-the-specsync-6-0-0-p1-release-defects-found-in-overnight-proving
artifact: design
---

# Design

Local, fail-closed fixes. No new verbs.

- Bundle `lifecycle-validation-limits.json` under `src/` so crates.io's include set can compile `include_str!`. Keep the CI copy byte-identical; a unit test pins that.
- `new` calls `generator::generate_spec` and `validate_scaffold_module_name` instead of a four-section private skeleton and `chrono_lite_today`.
- JSON parse failure in `load_json_config` sets `load_error` with the same wording as an unreadable file. `load_config` already refuses via `refuse_unloadable_config`.
- One `git_cmd(root)` helper: `env_clear`, PATH/locale/home/temp allowlist, `GIT_CEILING_DIRECTORIES` = parent of canonical root. Every production git spawn in `git_utils` uses it.
- `canonical_definition_artifact_payload` folds `\r\n` → `\n` (lone `\r` stays) before the tasks.md checkbox rewrite, matching `canonical_delta_body`.
- Missing `verification-attempts.json` is treated as an empty ledger at accept; a Verifying change refuses to recreate a missing ledger.
- `blank_fenced_code` space-blanks fenced lines at the original byte length so `--fix` heading offsets still index the source. Symbol readers and `--fix` `###` scans run on the blanked text.
- `check --spec NAME` is a clap flag merged with positional SPEC in `main`.
- `GenerationOutcome.skipped_no_files`; generate exits 1 when generated=0 and that list is non-empty.
- `scaffold --dir` is confined with `confined_generation_path`.
