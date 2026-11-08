---
change: make-merge-driver-installation-match-contracts-when-the-specs-directory-is-the-repository-root
artifact: testing
---

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
