# Lesson bundle: a-path-under-an-exclude-dirs-entry-is-not-source-for-the-verification-ignored-paths-overlap-rule

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: A path under an exclude_dirs entry is not source for the verification_ignored_paths overlap rule
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/verification_ignored_paths.rs, src/change_tests/verification_ignored_paths.rs, specs/change/, site/src/content/docs/configuration.md, CHANGELOG.md
- **Acceptance**: (1) With code at the root (source_dirs = ["."]) and exclude_dirs = [".chough"], verification_ignored_paths entries .chough/sessions/, .chough/archive/ and .chough/bugs/ are accepted, and verification ignores those folders. (2) An entry whose excluded component differs from the exclude_dirs name in case, or whose excluded name lies above the source_dirs root it is under, is still refused for overlapping source_dirs. (3) Every other overlap rule still refuses, even for an entry under an excluded name: .specsync/, the specs tree, schema_dir, meaningful_paths, spec files:, depends_on, Consumed By, the policy files, the principles file and the templates.

## Evidence

- Verification commit: `a5b8752e0e73da54800312c00cc2d0e5295c86ce`
- Base commit: `2cc0897fc00eee7db2b07f5134ba05639af39018`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #1083, from CorvidLabs/chough#251 and #279. CorvidLabs/NFTMix keeps its specified code at the repository root (`source_dirs = ["."]`), so there is no code folder to narrow to. Adding `.agents` and `.chough` to `exclude_dirs` restores `--require-coverage 100`, but listing Chough's folders in `verification_ignored_paths` is still refused: "verification_ignored_paths entry `.chough/sessions/` overlaps source_dirs `.`". The Chough tester reproduced it on NFTMix e60625a.

Leif's decision, 2026-11-18, relayed by hc t4c and posted on the issue: "build now. A path under an `exclude_dirs` entry counts as outside `source_dirs` for the `verification_ignored_paths` overlap rule. Failing test first." He approved the definition (ss-lead/gates/1083-definition.md) "Approve as written" on 2026-11-19 at about 14:19Z, relayed by hc t4c through ss-lead hc t58.

## From the change's design.md

# Design

- `exclude_dirs` entries are names, not paths. The coverage source walk skips every child whose name equals one exactly (`coverage_name_is_excluded`), and only while descending below a `source_dirs` root.
- So the overlap rule asks one question for the `source_dirs` scope only: does a component of the entry, below the `source_dirs` entry it lies under, equal an `exclude_dirs` name exactly? If so the walk never reaches anything under the entry, and the entry does not overlap that source scope.
- The prefix up to the source root is compared ignoring ASCII case, as the overlap test compares it. The excluded component is compared exactly, as the walk compares it: `.Chough/sessions/` stays refused when `.chough` is excluded.
- An excluded name at or above a `source_dirs` root excludes nothing below it (`source_dirs = [".chough/tools"]` keeps `.chough/tools/cache/` refused), because the walk starts at the root.
- Every other protected scope (`.specsync/`, the specs tree, `schema_dir`, `meaningful_paths`, spec `files:`, `depends_on`, `Consumed By`, the policy files, the principles file and the templates) is unchanged, and the scopes keep their order, so each existing refusal names the same rule.
- `exclude_patterns` is not counted, and `meaningful_paths = ["."]` still blocks the list (the definition's section 6).

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-115 | `src/change_tests/verification_ignored_paths.rs`: `an_entry_under_an_excluded_directory_does_not_overlap_source_dirs` (NFTMix's three entries accepted; `.Chough/sessions/` and `.chough/tools/cache/` under `source_dirs = [".chough/tools"]` refused), `every_other_overlap_rule_still_refuses_under_an_excluded_name` (`.specsync/`, specs tree, `files:`, principles file, template and `meaningful_paths` each still named), `a_root_code_project_lists_chough_s_folders_and_verification_ignores_them` (approve, check and status through the lifecycle `change check` runs), and every existing test in the file unchanged |

## How it runs

Unit tests, red on main 2cc0897f and then green; the full verify lane; and the real-repository run on NFTMix's shape.

## Where these lessons go

- `specs/change/change.3md`
