---
change: close-the-specsync-6-0-0-p1-release-defects-found-in-overnight-proving
artifact: research
---

# Research

- Cargo.toml `include = ["/src/**", …]` is why `.github/scripts/…` cannot be `include_str!`'d in the published crate. Duplicating the JSON under `src/` is the smallest fix that keeps CI and the binary on the same bytes.
- `load_config` already called `refuse_unloadable_config`; only JSON parse-fail skipped `load_error`. TOML parse-fail was fixed in #583.
- `verification_adopted` is true only when finalize adopts a moved HEAD. Deleting the ledger used to skip the empty-attempts check that lived inside `exists()`.
- `blank_fenced_code` that drops fenced lines preserves line count but not byte offsets. `--fix` uses `find_iter` byte offsets from the blanked text as indexes into the original, so blanking must be space-padded to the original length.
- `GIT_CEILING_DIRECTORIES` set to the parent of the passed root stops walk-up from an MCP snapshot sitting inside a host worktree. `is_git_repo` on a subdirectory of a repo therefore no longer walks to the parent `.git`; callers pass the project root.
