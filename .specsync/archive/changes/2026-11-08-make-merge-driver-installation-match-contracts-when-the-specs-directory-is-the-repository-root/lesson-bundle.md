# Lesson bundle: make-merge-driver-installation-match-contracts-when-the-specs-directory-is-the-repository-root

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Make merge driver installation match contracts when the specs directory is the repository root
- **Kind**: BugFix
- **Specs**: cmd_merge
- **Paths**: src/commands/merge.rs, specs/cmd_merge
- **Acceptance**: For specs_dir = . and ./, merge --install writes rules that git check-attr resolves as merge=specsync for root and nested .spec.md and requirements.md paths.
- **Acceptance**: The git check-attr regression test fails on the unmodified base before the fix and passes afterward; existing non-root install, quoting, symlink refusal, and rollback behavior stays unchanged.
- **Acceptance**: The change traces to MERGE-1 without new HI wording and adds one cmd_merge requirement assigned by the orc.

## Evidence

- Verification commit: `78f28ec4f4478eebb8fc6033f9a82c8c504ba0e3`
- Base commit: `b3ca5f97aa9c07b09ab6bfd8d7eece28f2c30f79`
- Verified by: `specsync check --spec cmd_merge`

## From the change's context.md

# Context

Issue #871 follows the #865 merge-driver review. At base `b3ca5f97aa9c07b09ab6bfd8d7eece28f2c30f79`, a repository with `.specsync/config.toml` containing `specs_dir = "."` runs `merge --install` successfully, but the generated `./**` attribute rules match neither root nor nested contracts. `git check-attr` reports `merge: unspecified` for root and nested `.spec.md` and `requirements.md` paths. The manual reproduction is Chough observation `OBS-product-test-b47ebf5a0d626500e6c08d9c` and blocking bug `BUG-merge-install-ignores-a-root-specs-directory-871-408287a7fe659cca53621579`.

The orc confirmed on 2026-11-06 that this traces to existing MERGE-1 with no new HI wording. The requirement belongs to `cmd_merge`, whose spec owns `src/commands/merge.rs` and installation behavior; the orc's `REQ-merge-007` reservation for a requirement in `specs/merge` is therefore not used. Scope approval waits for the orc's module-correct requirement ID.

The implementation is deliberately small: normalize `.` and `./` to the repository root and emit the same `**/*.spec.md` and `**/requirements.md` rules without a directory prefix. Keep existing normalization for non-root directories, wildcard escaping, C-style quoting, local driver configuration, symlink refusal, and failure rollback unchanged. A broader installer rewrite is out of scope.

Relative size: small, one normalization branch plus one behavioral regression test and contract updates. Chough timing has too few bug-fix and human-gate samples for a historical estimate, so no time range is claimed.

## From the change's testing.md

# Testing

## Plan

Add `root_specs_dir_matches_contracts` beside the existing install tests in `src/commands/merge.rs`. For both `.` and `./`, write the generated rules into a real temporary Git repository and assert that `git check-attr merge` returns `merge: specsync` for:

- `root.spec.md`
- `nested/root.spec.md`
- `requirements.md`
- `nested/requirements.md`

Run the new test before changing `attribute_lines`; it must fail on the unmodified base. After the normalization fix, run it again plus all `install_tests` to cover non-root paths, wildcard escaping, shell quoting, symlink refusal, and rollback.

Exercise the original command path in a fresh temporary repository with canonical `.specsync/config.toml`: run the rebuilt `target/release/specsync merge --install`, then check the same four attributes with Git. Finally run `fledge lanes run verify`, `fledge lanes run pre-push`, and the archive-tip GitHub CI lane.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-cmd-merge-004 | `root_specs_dir_matches_contracts` (red before the fix, green afterward), existing `install_tests`, and the manual `merge --install` plus `git check-attr` reproduction |

## Results

Red first on base `b3ca5f97` plus only the new test: `cargo test --bin specsync root_specs_dir_matches_contracts` failed at `root.spec.md` with `merge: unspecified`; 1 test failed and 2,898 were filtered out. This matches the earlier manual failure in `OBS-product-test-b47ebf5a0d626500e6c08d9c`. No fix was present in that run.

The first implementation handled an empty prefix after `./` but missed bare `.`. The focused suite stayed red at the same assertion, with 5 passing and 1 failing. After explicit bare-dot normalization, `cargo test --bin specsync install_tests` passed all 6 tests and filtered out 2,893.

With the rebuilt release binary in a fresh repository using canonical `.specsync/config.toml` and `specs_dir = "."`, `merge --install` emitted `**/*.spec.md` and `**/requirements.md`. `git check-attr` returned `merge: specsync` for `root.spec.md`, `nested/root.spec.md`, `requirements.md`, and `nested/requirements.md`. Chough observations: `OBS-product-test-8b3cf28186053c41615bf658` and `OBS-product-test-e62f4a4b773df570bf80de09`.

`fledge lanes run verify` was interrupted by the orc's SAFE-PARK instruction and exited 130. It has no pass/fail verdict and must be rerun on resume; the interrupted receipt is `OBS-product-test-f6bd8d6b1f50fef57acf10bb`. The final verify/pre-push task remains open.

## Where these lessons go

- `specs/cmd_merge/cmd_merge.3md`
