---
change: CHG-0108-stop-reporting-success-for-checks-that-did-not-happen-gate-drafts-that-document
artifact: tasks
---

# Tasks

## Quoting (REQ-parser-002, REQ-hash-cache-002)

- [x] Add `unquote_yaml_scalar` with comment handling and unterminated-quote errors
- [x] Apply to block list items
- [x] Apply to scalar values, passing `[`-prefixed values through untouched
- [x] Mirror the unquoting in `hash_cache::extract_frontmatter_files`

## Cold cache (REQ-hash-cache-002, REQ-cmd-check-005)

- [x] Add `HashCache::has_baseline`
- [x] Add `ChangeClassification::baseline_known` and `::reportable`
- [x] Switch the requirements and companion reporting sites to `reportable`
- [x] Confirm selection is unchanged — a cold cache still re-validates

## Draft gate (REQ-commands-006, REQ-types-005, REQ-validator-011)

- [x] Add `had_present_source` and `documents_contract` to `ValidationResult`
- [x] Set `had_present_source` in both present-file branches only
- [x] Set `documents_contract` from `get_spec_symbols`, before the section gates
- [x] Warn on the conjunction in `commands/mod.rs`
- [x] Confirm the three pinned draft contracts pass **unedited**

## Remediation (REQ-change-064)

- [x] Add `UNCOVERED_PATH_FLAG_LIMIT` and the remainder summary

## Verification

- [x] `cargo fmt --all -- --check` clean
- [x] `cargo clippy -- -D warnings` exit 0
- [x] Full `cargo test` green — 2210 unit, 331 integration, 0 failures
- [x] All four fixes re-verified by hand against the fixtures that exposed them
