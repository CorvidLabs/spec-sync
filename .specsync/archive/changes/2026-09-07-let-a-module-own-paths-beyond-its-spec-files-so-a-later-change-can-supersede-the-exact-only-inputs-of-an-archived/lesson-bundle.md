# Lesson bundle — let-a-module-own-paths-beyond-its-spec-files-so-a-later-change-can-supersede-the-exact-only-inputs-of-an-archived

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Let a module own paths beyond its spec files so a later change can supersede the exact-only inputs of an archived bootstrap change
- **Kind**: Feature
- **Specs**: change, config, types
- **Paths**: src/change.rs, src/change_tests.rs, src/config.rs, src/types.rs, specs/change/context.md, specs/change/tasks.md, specs/change/testing.md, specs/config/context.md, specs/config/tasks.md, specs/config/testing.md, site/src/content/docs/configuration.md, site/src/content/docs/cli.md
- **Acceptance**: `[modules."<name>"] owns` in `.specsync/config.toml` gives a declared module paths beyond its spec's `files:` — a file, or a directory that owns everything beneath it — and an acceptance manifest signs a matching path (a directory entry included) under the module instead of `@exact:test` or `@exact:delivery`; the key is not a source mapping, `specsync check` demands no spec coverage for it, and nothing under `.specsync/` can be owned
- **Acceptance**: `change supersede --spec <module>` accepts a predecessor entry whose signed owners are all reserved exact labels when the module owns the path under the current configuration, refuses it otherwise naming the frozen label and the `owns` remedy without persisting anything, and still refuses a module that is not a signed owner of an entry a module signed
- **Acceptance**: The successor walk covers a changed exact-only predecessor entry through any authenticated successor that declared it, judged by every check a signed owner's successor passes; the succession tuple is unchanged (successor module, predecessor entry digest, successor entry digest, digest-matches-base-tree) and no reopen, owner correction, or additional audit record is needed; a workflow-v2 successor that edits, deletes, and re-signs such inputs finalizes, and the bootstrap is successor-covered on `check_project` and `audit_project` before and after the archive commit; the exact-only diagnostic names the supersede alternative beside the audited reopen wherever the configuration can grant the path
- **Acceptance**: Regression tests: `configured_module_ownership_lets_a_v2_successor_supersede_exact_only_inputs_of_a_bootstrap_change` is refused on 404fe4d6 and passes with the feature; `supersede_refuses_an_exact_only_input_the_configuration_grants_no_module` and `configured_ownership_overrides_reserved_exact_classes_for_declared_modules_only` hold; `fledge run lint` and `fledge lanes run verify` pass

## Evidence

- Verification commit: `d9a3190cdd1b7eb356d6c5c9ef98071d1e6f12d6`
- Base commit: `404fe4d6fcef380d3675bab5cc1d2d4786d0401c`
- Verified by: `specsync check --spec change --spec config --spec types`

## From the change's context.md

# Context

Field-verified on CorvidLabs/swift-algorand (PRs #17–#20, four workflow-v2 changes archived in a stack). Acceptance manifests give module ownership (`owners: ["algorand"]`) only to the paths a spec's `files:` frontmatter lists. Everything else under a change's declared paths is signed `@exact:test` (tests, fixtures) or `@exact:delivery` (`Package.swift`, DocC catalog pages under `Sources/`, `fledge.toml`, `.github/`), and an exact-only input cannot be superseded: `change supersede … --spec algorand` refuses with "module `algorand` is not a successor-eligible signed owner of predecessor path …", `@exact:delivery` is not a module name, and the walk's only remedy for a changed exact-only input is an audited reopen of the predecessor. In a repository bootstrapped by a whole-tree governance change (`CHG-0001` declared `Sources/`, `Tests/`, `Package.swift`, `.github/`, `specs/`), every non-spec file is therefore frozen for every later change: no successor can edit or delete a legacy test file, gate a dependency in `Package.swift`, or fix a DocC page. Reopening the bootstrap replays its canonical delta over the successors' materialization (its `## ADDED` section block conflicts, its `## MODIFIED` REQUIREMENT blocks overwrite amended text), and `correct-owner` requires a reopened predecessor AND a module that currently owns the path — which, for tests, none does.

Concrete casualties: a Swift Testing port of 14 legacy XCTest files; removal of the deprecated trapping APIs those files use; DocC plugin gating in `Package.swift` (parked, verified, on swift-algorand branch `feat/docc-gating` @ adeeeec, carved out of PR #20 together with the DocC pages); CI trigger changes under `.github/`.

Maintainer's decision: extend spec-sync so a module can own paths beyond its spec's `files:`. Two design points are settled in `design.md`: (A) the declaration and its reach, and (B) succession against history, whose manifests are frozen with `@exact:*` owners.

Constraints a resuming session needs: predecessor manifests are immutable and their owner labels are never rewritten; the succession tuple domain (`specsync.semantic-succession.v1`) and the acceptance entry/manifest digests are unchanged; `specsync check` must not start demanding spec coverage for owned paths; `.specsync/` must never become configurable ownership, because the sequence-ledger succession rule reads its exact owners; a spec's `files:` list must keep leaving a mapped test exact-only (`mapped_tests_remain_exact_only`). Ruled out: a new ownership-transfer audit record (the two signed manifests and the tuple between them already prove who owned the path when); globs in `owns` (a second matching vocabulary inside acceptance evidence, when `affected_paths` already has one); retiring a signed module owner's claim by configuration (an entry a module signed still wants that module's successor — ownership migration between modules is a separate question, left open).

Based on #753's branch (`0xleif/fix/archive-preflight-authenticates-v2-successor`, 404fe4d6) rather than `main`, because the regression scenario finalizes a workflow-v2 successor over a legacy accepted change, which needs #753's preflight and audit fixes. Related: #753, #751, #688.

## From the change's design.md

# Design

## A. Declaration — `[modules."<name>"] owns`

- `owns = ["Tests/AlgorandTests", "Package.swift", "Sources/Algorand/Algorand.docc"]` under the existing `[modules."<name>"]` table; `ModuleDefinition` gains `owns: Vec<String>` beside `files` and `depends_on`. Parsed by `parse_toml_modules_nested`, typed as a string array by the checked parser, written by `config_to_toml`; a module carrying only `owns` still round-trips. Omitted, everything behaves as today.
- An entry is a project-relative file, or a directory that owns everything beneath it, matched by `path_matches_scope` — the vocabulary `affected_paths` already uses. Not globs: `exclude_patterns` globbing lives in the coverage scanner, and acceptance evidence should not acquire a second matching semantics that a manifest reader then has to reproduce.
- Reach: `acceptance_input_owners` consults `owns` for the modules the change DECLARES, ahead of the reserved exact classes. An owned test, fixture, or delivery path is signed under the module instead of `@exact:test` / `@exact:delivery`; a path no declared module owns keeps its exact class; a spec's `files:` list still does not lift a mapped test out of `@exact:test`. Nothing else reads the key: it is not a source mapping, so `specsync check`, coverage, and `find_files_for_module` ignore it and demand no spec coverage for an owned path.
- `.specsync/` and the protected SDD paths are never configurable ownership (`ownership_is_configurable`): the sequence-ledger succession rule reads their exact owners, and the lifecycle's own ledger must not be re-homed by the project it governs.
- A directory entry (`kind: non_file`) is treated exactly like a file: it takes configured ownership, and it needs succession only when it changes.

## B. Succession against history

- Predecessor manifests are immutable and still say `@exact:test`; nothing rewrites them. A reserved exact owner is not a module's claim — it records that no module owned the input when it was signed — so it is retired by whichever module owns the path NOW. `validate_supersedes_semantics` (draft `supersede` and acceptance) accepts a module for an entry whose signed owners are all exact when `module_currently_owns_path` says so, by the same `acceptance_input_owners` rule that will sign the successor's own entry; it refuses otherwise, naming the frozen label and the `owns` remedy. An entry a module signed keeps the historical rule: a module that is not among its signed owners is still refused, so a signed owner's claim is never retired by configuration.
- The succession tuple is unchanged: `(predecessor_id, path, module, predecessor_entry_digest, successor_entry_digest)` under the same domain, with `module` the successor's module and the digest-matches-base-tree rule intact. The successor's signed manifest carrying the path under that module is what proves the module owned the path when it superseded it.
- The walk reads the claimants of a changed exact-only entry from the successors' declared obligations (`succession_claimants`) instead of from the entry — the entry names nobody — and judges each claimant with `successor_covers_input`, the same checks a signed owner's successor passes (declared obligation, authenticated evidence, resolved manifest, matching tuple, manifest carries the successor entry under the module, non-removed semantic item, tuple holds, recursive freshness). One authenticated claimant covers the entry; otherwise every refused claimant is named with its reason, with the frozen label as the input's owner. A signed module owner's entry keeps per-owner coverage: every historical owner still needs its own successor.
- No extra audit record for `@exact:test` → module succession: the two signed manifests and the tuple between them already prove who owned the path when, and a third ledger would only be one more thing to authenticate. `correct-owner` is not involved.
- The exact-only diagnostic keeps the audited-reopen remediation and, wherever the configuration can grant the path, names the supersede alternative — the moment the tool knows the remedy is the moment it is cheapest to say.

## From the change's testing.md

# Testing

- `configured_module_ownership_lets_a_v2_successor_supersede_exact_only_inputs_of_a_bootstrap_change` — DISCRIMINATOR. A legacy (v1) bootstrap declares `tests/auth`, `Package.swift`, and `src/auth.rs`, is accepted and committed before any `owns` exists; its manifest signs `tests/auth/legacy.rs`, `tests/auth/deprecated.rs`, and the `tests/auth` directory entry (`non_file`) `@exact:test` and `Package.swift` `@exact:delivery`. Before the configuration grants the paths, `add_supersedes_obligation` refuses `auth` with the new message and persists nothing (on 404fe4d6 the refusal is "not a successor-eligible signed owner"). With `[modules."auth"] owns = ["tests/auth", "Package.swift"]`, a workflow-v2 successor adopts all five entries, edits the test, deletes the deprecated test, edits the package manifest, and goes approve → check → commit → check → review → finalize. Its manifest signs every owned path under `auth` (the directory entry and the `missing` deletion included), its tuples bind each frozen predecessor digest to `auth`, and the bootstrap is `SuccessorCovered` on `check_project` and `audit_project` before and after the archive commit. A later edit of the owned test is reported through the archived successor with its reason and does not offer the legacy reopen.
- `supersede_refuses_an_exact_only_input_the_configuration_grants_no_module` — NEGATIVE. `owns = ["tests/auth"]` only: the test file is adopted, `Package.swift` is refused naming `@exact:delivery` and the `owns` remedy, the persisted record carries exactly the one obligation, and `src/auth.rs` (a module-signed entry) still resolves to `auth` by the historical rule.
- `configured_ownership_overrides_reserved_exact_classes_for_declared_modules_only` — unit CONTROL over `acceptance_input_owners`: a configured test file, the directory itself, and root delivery metadata resolve to the declared module; an undeclared module's paths, an unowned test tree, `fledge.toml`, and everything under `.specsync/` keep their exact class.
- `test_module_owns_alone_round_trips_and_is_not_a_files_mapping`, `test_config_to_toml_roundtrips_modules` — config round-trip with and beside `files`.
- `stale_accepted_change_error_names_exact_only_input_and_audited_reopen` — pins the amended exact-only message; `mapped_tests_remain_exact_only`, `finalize_archives_a_v2_successor_that_supersedes_a_legacy_accepted_change`, and the `stale_accepted_change_error_names_*` family are unchanged CONTROLS.
- `fledge run lint` (clippy, `-D warnings`), `fledge lanes run pre-push`, `fledge lanes run verify`.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-change-095 | `configured_module_ownership_lets_a_v2_successor_supersede_exact_only_inputs_of_a_bootstrap_change`, `supersede_refuses_an_exact_only_input_the_configuration_grants_no_module`, `configured_ownership_overrides_reserved_exact_classes_for_declared_modules_only` |
| REQ-change-020 | `configured_module_ownership_lets_a_v2_successor_supersede_exact_only_inputs_of_a_bootstrap_change`, `supersede_refuses_an_exact_only_input_the_configuration_grants_no_module` |
| REQ-change-024 | `configured_module_ownership_lets_a_v2_successor_supersede_exact_only_inputs_of_a_bootstrap_change` |
| REQ-change-036 | `stale_accepted_change_error_names_exact_only_input_and_audited_reopen`, `configured_module_ownership_lets_a_v2_successor_supersede_exact_only_inputs_of_a_bootstrap_change` |
| REQ-config-013 | `test_module_owns_alone_round_trips_and_is_not_a_files_mapping`, `test_config_to_toml_roundtrips_modules` |

## Where these lessons go

- `specs/change/context.md`
- `specs/config/context.md`
- `specs/types/context.md`
