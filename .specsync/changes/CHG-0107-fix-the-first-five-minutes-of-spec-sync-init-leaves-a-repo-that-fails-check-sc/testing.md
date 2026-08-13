---
change: CHG-0107-fix-the-first-five-minutes-of-spec-sync-init-leaves-a-repo-that-fails-check-sc
artifact: testing
---

# Testing

## Strategy

Each fix removes a finding that should never have fired. The risk is therefore not that
the fix fails to work but that it removes findings that *should* fire. Every mechanism
consequently gets a **negative test that is the load-bearing one** — the assertion that
the guard still bites where it must.

The Rust suite is single-process and single-root, so it cannot observe squash merges,
multi-clone races, or branch topology. The sandbox drills judge those. Drill 038 (the
drift invariant) must stay 10/10 and drill 039 (export extraction, seven languages)
43/43; if either moves, the fix broke the product.

## Tests added

### Bootstrap record

| Test | Asserts |
|---|---|
| `fresh_init_leaves_the_next_lifecycle_check_clean` | `git init` → `specsync init` → check is clean, both **before** the bootstrap commit (the one-commit `HEAD~1` case) and after; filling in `verification_commands` does not revoke the bootstrap; **widening `meaningful_paths` does revoke it** |
| `bootstrap_records_exempt_only_newly_created_protected_paths` | A record cannot exempt a path that is not protected, nor one already present at the comparison base |
| `uncovered_paths_error_names_the_escape_hatch_and_ignore_precedence` | The remaining uncovered-paths error stays actionable |

The revocation assertion is the important one: it proves the exemption is a pin on the
enforcement surface rather than a blanket amnesty for anything under `.specsync/`.

### Stub exemption

| Test | Asserts |
|---|---|
| `effective_contract_exempts_stub_sections_no_active_change_authored` | The positive case — scaffold output passes |
| `effective_contract_keeps_authored_emptied_section_fatal` | **The negative case** — a section an active change authored and then emptied stays fatal |
| `effective_contract_keeps_applied_authored_emptied_section_fatal` | Same, through the applied-delta path |
| `effective_contract_exempts_nothing_when_authorship_is_unknown` | Fails closed when authorship cannot be determined |
| `effective_contract_reports_ignore_rule_suppressions` | Ignore-rule suppressions are surfaced, not silent |
| `effective_contract_reports_suppressions_alongside_errors` | Suppressions do not mask concurrent errors |
| `project_check_reports_effective_contract_suppressions_as_warnings` | Suppressions reach `check` output as warnings |

Failing closed on unknown authorship matters: the exemption's key is "no active change
authored this", and an inability to answer that question must not be read as "no".

### Directory mappings

| Test | Asserts |
|---|---|
| `directory_source_mapping_fails_loud_and_names_the_files_to_list` | The error fires and the fix names the expanded files |
| `directory_source_mapping_does_not_disturb_sibling_file_mappings` | Valid file entries in the same `files:` block are unaffected |
| `snapshot_validation_reports_a_directory_mapping_as_a_directory` | The snapshot path reports a directory, **not** an out-of-root escape |
| `draft_directory_source_mapping_is_not_a_planned_mapping_notice` | A directory is not misreported as a planned-but-absent mapping |

## Results

- `cargo fmt --check` — clean
- `cargo clippy -- -D warnings` — exit 0
- `cargo test` — **2203 unit, 331 integration, 0 failures**
- Sandbox board at the base commit — **28 pass, 0 fail, 0 skip**; drill 039 43/43

## Not covered here

Behavior of a repository initialized by an earlier 6.0 build that has no bootstrap
record. Reasoned rather than tested: the exemption only ever removes a finding, so its
absence reproduces today's behavior exactly. 6.0 is untagged, so no released version is
affected.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-cmd-init-005 | `cargo test`; `fresh_init_leaves_the_next_lifecycle_check_clean` asserts `.specsync/bootstrap.json` exists after `cmd_init` and that the next check is clean both before and after the bootstrap commit. Confirmed by hand: `git init` → `specsync init` → `specsync check` exits 0 on a repository with one commit and an uncommitted `.specsync/` tree |
| REQ-change-060 | `cargo test`; `bootstrap_records_exempt_only_newly_created_protected_paths` proves a record cannot exempt a non-protected path nor one already present at the comparison base, and `fresh_init_leaves_the_next_lifecycle_check_clean` proves that widening `meaningful_paths` revokes the exemption the edited file was granted |
| REQ-change-061 | `cargo test`; `fresh_init_leaves_the_next_lifecycle_check_clean` populates `verification_commands` and asserts the exemption survives, then edits `meaningful_paths` and asserts it is revoked — the two halves of the enforcement-surface projection |
| REQ-change-062 | `cargo test`; the same test checks a one-commit repository before the bootstrap commit exists, the case where `HEAD~1` does not resolve |
| REQ-change-063 | `cargo test`; `effective_contract_exempts_stub_sections_no_active_change_authored` covers the positive case, `effective_contract_keeps_authored_emptied_section_fatal` and `effective_contract_keeps_applied_authored_emptied_section_fatal` the negative case through both delta paths, `effective_contract_exempts_nothing_when_authorship_is_unknown` the fail-closed case, and `effective_contract_reports_ignore_rule_suppressions`, `effective_contract_reports_suppressions_alongside_errors`, and `project_check_reports_effective_contract_suppressions_as_warnings` the reporting of suppressions. Confirmed by hand: `specsync scaffold auth` → `specsync check` exits 0 |
| REQ-validator-010 | `cargo test`; `directory_source_mapping_fails_loud_and_names_the_files_to_list` asserts the error and the expanded fix, `directory_source_mapping_does_not_disturb_sibling_file_mappings` that valid entries in the same block are unaffected, and `draft_directory_source_mapping_is_not_a_planned_mapping_notice` that a directory is not misreported as a planned mapping |
| REQ-cmd-issues-002 | `cargo test`; `snapshot_validation_reports_a_directory_mapping_as_a_directory` asserts the snapshot path reports a directory rather than an out-of-root escape, with symlink rejection still evaluated first |
