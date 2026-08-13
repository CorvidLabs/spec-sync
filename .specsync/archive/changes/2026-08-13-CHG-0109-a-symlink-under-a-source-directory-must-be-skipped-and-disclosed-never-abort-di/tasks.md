---
change: CHG-0109-a-symlink-under-a-source-directory-must-be-skipped-and-disclosed-never-abort-di
artifact: tasks
---

# Tasks

## Collect instead of abort (REQ-validator-012)

- [x] Add `skipped_links: BTreeSet<String>` to `CoverageTraversalBudget`
- [x] Add `record_skipped_link` with slash normalization and dedup
- [x] Convert `retained_source_dirs_by_scan` to record-and-continue
- [x] Convert `retained_directory_contains_source` to record-and-continue
- [x] Convert `snapshot_coverage_directory` to record-and-continue
- [x] Leave the configured `source_dirs` site fatal, with the reason in a comment
- [x] Leave the spec-tree sites fatal

## Carry and disclose (REQ-types-006, REQ-output-003, REQ-cmd-check-006)

- [x] Add `CoverageReport::skipped_links`, populated from the budget
- [x] Report `Vec::new()` on the inconclusive-coverage fallback
- [x] Text disclosure via `print_skipped_links`, from `print_coverage_line`
- [x] Markdown disclosure inside the coverage section
- [x] JSON `skipped_links` array carrying the full list

## Gate (REQ-commands-007)

- [x] `compute_exit_code` — strict returns 1 when links were skipped
- [x] `exit_with_status` — the same, naming the count
- [x] Confirm bare `check` still exits 0

## Verification

- [x] `cargo fmt --all -- --check` clean
- [x] `cargo clippy -- -D warnings` exit 0
- [x] `cargo test` green — 2210 unit, 331 integration, 0 failures
- [x] 48 symlink tests still pass, including the escape guard
- [x] Hand-verified: link inside root, link escaping the root, JSON, markdown, `--strict`
- [x] Sandbox drill 040 extended and proven to discriminate (21/21 vs 10/21)
