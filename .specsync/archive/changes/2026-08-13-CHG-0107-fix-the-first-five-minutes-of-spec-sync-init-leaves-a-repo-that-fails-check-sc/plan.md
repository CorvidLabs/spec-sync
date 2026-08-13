---
change: CHG-0107-fix-the-first-five-minutes-of-spec-sync-init-leaves-a-repo-that-fails-check-sc
artifact: plan
---

# Plan

The implementation was completed and the full suite run **before** `change new`, per
#542: delivery scope freezes at the interview and cannot be widened, while blast radius
only becomes visible at compile, test, and verification time. The declared scope is
therefore measured, not estimated.

## Sequence

1. **Bootstrap record.** Add `BOOTSTRAP_RECORD_PATH` and `BOOTSTRAP_RECORD_CANDIDATES`
   to `change`; implement `record_bootstrap_paths`, `bootstrap_exempt_paths`,
   `bootstrap_records`, `bootstrap_record_entry`, `bootstrap_digest`,
   `policy_enforcement_projection`, `content_digest`, and the four predicate helpers
   (`comparison_base_commit`, `commit_is_ancestor_of_head`, `project_path_is_absent_at`,
   `project_path_matches_digest`). Call it from `execute_init`, collecting failure as a
   warning.
2. **Comparison base.** Replace the `HEAD~1...HEAD` fallback with
   `comparison_base_commit`, reducing both diff-base forms to a merge base with `HEAD`.
3. **Stub exemption.** Add `STUB_SECTION_WARNING_PREFIX`; teach
   `validate_effective_contracts` to exempt stub warnings for sections no active change
   authored, routing through `IgnoreRules`.
4. **Directory mappings.** Add `SourceSnapshot::Directory`; branch on it in
   `snapshot_source_file`; add the `directory_mapping` branch to
   `validate_spec_content_internal`; implement `directory_mapping_fix` and
   `expand_directory_mapping`; make `generator::find_module_source_files` public.
5. **Verify.** `cargo fmt --check`, `cargo clippy -- -D warnings`, full `cargo test`.
6. **Author the change workspace** with the measured scope; update the five affected
   canonical specs to document the two new public functions and the new requirements.
7. **`change check --commit`**, then ship and open the PR.

## Ordering constraints

- Step 4 must precede any run of `check` against this repository: making
  `find_module_source_files` public without documenting it in `generator.spec.md`
  produces the very undocumented-export drift this tool exists to catch. The same
  applies to `record_bootstrap_paths` in `change.spec.md`.
- Steps 1 and 3 both feed `check_project`. They must be verified together, not
  separately — a fresh `init` → `scaffold` → `check` sequence exercises both at once.

## Rollout

Lands before the `v6.0.0` tag, so no migration is owed. Repositories initialized by
earlier 6.0 builds have no bootstrap record; they are unaffected, because the exemption
only ever removes a finding and never adds one.
