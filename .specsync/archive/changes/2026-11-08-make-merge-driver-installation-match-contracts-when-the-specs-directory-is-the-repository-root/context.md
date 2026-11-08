---
change: make-merge-driver-installation-match-contracts-when-the-specs-directory-is-the-repository-root
artifact: context
---

# Context

Issue #871 follows the #865 merge-driver review. At base `b3ca5f97aa9c07b09ab6bfd8d7eece28f2c30f79`, a repository with `.specsync/config.toml` containing `specs_dir = "."` runs `merge --install` successfully, but the generated `./**` attribute rules match neither root nor nested contracts. `git check-attr` reports `merge: unspecified` for root and nested `.spec.md` and `requirements.md` paths. The manual reproduction is Chough observation `OBS-product-test-b47ebf5a0d626500e6c08d9c` and blocking bug `BUG-merge-install-ignores-a-root-specs-directory-871-408287a7fe659cca53621579`.

The orc confirmed on 2026-11-06 that this traces to existing MERGE-1 with no new HI wording. The requirement belongs to `cmd_merge`, whose spec owns `src/commands/merge.rs` and installation behavior; the orc's `REQ-merge-007` reservation for a requirement in `specs/merge` is therefore not used. Scope approval waits for the orc's module-correct requirement ID.

The implementation is deliberately small: normalize `.` and `./` to the repository root and emit the same `**/*.spec.md` and `**/requirements.md` rules without a directory prefix. Keep existing normalization for non-root directories, wildcard escaping, C-style quoting, local driver configuration, symlink refusal, and failure rollback unchanged. A broader installer rewrite is out of scope.

Relative size: small, one normalization branch plus one behavioral regression test and contract updates. Chough timing has too few bug-fix and human-gate samples for a historical estimate, so no time range is claimed.
