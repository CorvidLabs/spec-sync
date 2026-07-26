---
spec: scoring.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/scoring.rs` | cargo test scoring:: | `test_count_placeholder_todos`, `test_count_placeholder_todos_in_code_blocks`, `test_count_placeholder_todos_zero`, `test_placeholder_comments_count_prose_only`, `test_count_sections_with_content`, `test_count_sections_with_content_stubs_not_counted`, `test_compute_project_score_distribution`, `test_score_spec_complete`, `test_score_spec_stub_sections_penalized`, `test_explain_has_all_dimensions` |
| #421 false-green matrix | `cargo test scoring::tests::` | `all_generator_placeholder_spec_does_not_score_a`, `all_todo_spec_stays_below_passing_bar`, `freshness_detects_source_newer_than_untracked_spec` |

## Coverage Gaps

- Integration gap: add a fixture for "Perfect spec" before changing user-visible CLI output, generated files, or error handling in scoring.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Perfect spec | a spec with complete frontmatter, all sections present, 100% API coverage, no unfinished-work markers, all files exist | `score_spec` is called | returns total=100, grade="A", empty suggestions |
| Skeleton spec with unfinished markers | a spec with all sections but only unfinished-work markers in content | `score_spec` is called | depth_score is low and suggestions identify the sections that need substantive content |
| Untouched scaffold | generator-owned guidance fills at least half the required sections | `score_spec` is called | total is below 80 |
| Untracked stale spec | source mtime is newer than spec mtime | `score_spec` is called | freshness loses 5 points with an actionable suggestion |
| Project score aggregation | 3 specs scoring 95, 80, 65 | `compute_project_score` is called | average_score=80.0, grade="B", distribution shows 1 A, 1 B, 0 C, 1 D, 0 F |
| --explain breakdown | a spec scoring 11/20 on Depth | `score_spec` is called | `explain` contains a Depth entry with `CriterionResult` items showing which checks passed/failed |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Spec file unreadable | Returns score=0, grade="F", suggestion: "Cannot read spec file" | Keep or add a focused assertion before changing this behavior |
| Missing frontmatter | Returns score=0, grade="F", suggestion: "Add YAML frontmatter" | Keep or add a focused assertion before changing this behavior |
| No spec files in project | `compute_project_score` returns average=0, grade="F" | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/scoring.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `./target/release/specsync score --all`.
