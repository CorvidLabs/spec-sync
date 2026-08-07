---
change: CHG-0089-add-change-check-commit-to-perform-the-sequence-it-requires
artifact: context
---

# Context

`change check` materializes the approved delta into the working tree and anchors
verification at the commit *before* that materialization. Committing afterward
stales the just-recorded evidence, so CI only accepts a second verification
against the committed tree. Agents and humans were rediscovering that loop on
nearly every PR (two red lifecycle gates, one cause).

`--commit` performs the full sequence: verify → commit materialize → re-verify →
commit evidence. `--push` is optional and refuses to act without `--commit`.

Dogfood: sandbox drill `035-check-commit.sh`. Implementation lives in
`src/commands/change.rs` (`run_checked_commit`) with CLI flags on `ChangeAction::Check`.
