---
change: close-the-specsync-6-0-0-p1-release-defects-found-in-overnight-proving
artifact: context
---

# Context

Overnight proving against `0d0251bf` produced 196 findings, 8 of them P1. Fixes were drafted in isolated worktrees that never reached GitHub; this package reconstructs them onto one branch and takes them through the 6.0 lifecycle.

## P1s closed here

1. `cargo publish` could not compile: `include_str!` of `.github/scripts/lifecycle-validation-limits.json` is outside Cargo.toml's `/src/**` include set.
2. `specsync new` emitted 4 of the 7 sections `init` configures.
3. Malformed JSON config silently fell back to defaults in `rules`/`rehash`/`compact`/`deps`/`archive-tasks`/`view` (#653, incomplete #583).
4. MCP/git children inherited `GITHUB_TOKEN` and `GIT_DIR`.
5. `ship`/`finalize` refused a CRLF checkout (post-move archive digest vs LF blob).
6. README / site quick start failed at the first `check --strict` after `add-spec` with no source.
7. Deleting `verification-attempts.json` bypassed the empty-ledger adoption guard (#656).
8. Public API readers counted fenced example backticks as documented exports; `--fix` used a fenced `###` as an insert target (#768.3).

Also closed on the same tree: `check --spec` as a real flag (evidence already recorded that form), `generate` silent no-op on empty file lists, `scaffold --dir` path escape.

## Constraints

- One change package, every touched `src/` path owned.
- Do not create tags, dispatch `promote`, `cargo publish`, touch Homebrew, force-push `main`, or merge the PR.
- `blank_fenced_code` must preserve byte length: `--fix` uses blanked offsets as indexes into the original.

## Out of scope

#605 coverage gate skip, #675/#690 messages, #439 performance, #434 unknown-field preservation, #532 multi-clone approvals, #768.1/#768.2 remaining `--fix` table-scanner cases, `report --require-coverage` stale skip.
