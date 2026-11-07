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
