---
change: run-clippy-on-every-target-in-ci-and-the-verify-lane-and-clear-the-test-code-lints-it-finds
artifact: tasks
---

# Tasks

- [x] Fix the 19 `field_reassign_with_default` sites with struct-update initializers
- [x] Fix the 5 `manual_contains` sites with `contains`
- [x] Fix the 2 `type_complexity` sites with `#[cfg(test)]` type aliases
- [x] Fix the 2 `items_after_test_module` sites by reordering `scaffold.rs` and `init.rs`
- [x] Fix the remaining 8 (`nonminimal_bool` ×2, `needless_return`, `unused_mut`, `collapsible_if`, `cloned_ref_to_slice_refs`, `bool_comparison`, and `assertions_on_constants` as a `const` assertion)
- [x] `cargo clippy --all-targets -- -D warnings` exits 0, with no new `allow` or `expect` attribute
- [x] CI's clippy step and the fledge `lint` task run `--all-targets`
- [x] `clippy-all-targets-test` runs in the fledge `ci`, `verify` and `repo` lanes and in `ci.yml`'s `validate-action` job
- [x] `specs/github`: REQ-github-028 materialized, version, testing row, Change Log row and `github.3md` lesson plane
