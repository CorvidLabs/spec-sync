---
change: a-path-under-an-exclude-dirs-entry-is-not-source-for-the-verification-ignored-paths-overlap-rule
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-115 | `src/change_tests/verification_ignored_paths.rs`: `an_entry_under_an_excluded_directory_does_not_overlap_source_dirs` (NFTMix's three entries accepted; `.Chough/sessions/` and `.chough/tools/cache/` under `source_dirs = [".chough/tools"]` refused), `every_other_overlap_rule_still_refuses_under_an_excluded_name` (`.specsync/`, specs tree, `files:`, principles file, template and `meaningful_paths` each still named), `a_root_code_project_lists_chough_s_folders_and_verification_ignores_them` (approve, check and status through the lifecycle `change check` runs), and every existing test in the file unchanged |

## How it runs

Unit tests, red on main 2cc0897f and then green; the full verify lane; and the real-repository run on NFTMix's shape.
