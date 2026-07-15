---
change: CHG-0043-make-accepted-change-validity-successor-aware-with-exact-per-input-evidence-rec
artifact: research
---

# Research

`acceptance_input_digest` already distinguishes content, symlink targets, Git modes, gitlinks, missing paths, non-files, post-delta canonical overrides, and historical sequence-ledger content. Its framing is secure, but its final aggregate discards the per-entry boundaries needed for partial successor reasoning.

`accepted_change_has_current_canonical_successors` currently unions affected specs and paths independently and admits `canonical_applied` records regardless of their current lifecycle state. `canonical_successor_governs_stale_predecessor` separately admits implementing and verifying candidates. Neither helper validates the same complete closing contract used by accepted project checks, and neither carries deterministic evidence that one semantic change actually transformed a predecessor entry.

`check_project`, `summarize_change`, `reopen_change`, and `archive_change` currently reach closing validity through different paths. Active listing excludes dated archive records even though `load_change` can locate them. These differences explain inconsistent status/reopen/archive behavior and why archiving a successor can remove it from later inference.

Current helpers construct artifact paths from the active workspace even after location discovery, so archived validation needs a first-class located-workspace handle rather than ID-only path recomputation. Archive also changes persisted state before moving the workspace, so preserving and authenticating the prior accepted projection is necessary to validate closing evidence immediately before the archive commit exists.

Canonical `REQ-change-024` currently promises that implementing and verifying successors can suppress a predecessor. That is incompatible with the fail-closed terminal-successor contract and must be modified together with `REQ-change-012`, `REQ-change-014`, `REQ-change-017`, `REQ-change-018`, and `REQ-change-020`.

The stability audit found that statting files after an index-only snapshot was insufficient: a caller could consume a different worktree generation, an alternate index could be ignored, and scanning every `sharedindex.*` file both over-scoped trust and permitted unrelated denial of service. Git's effective index path plus `--shared-index-path`, scoped stage output, and a repeated immutable candidate capture close those gaps without parsing index internals or assuming SHA-1 object widths.

The subprocess audit also found that `Command::output` and sequential stdin/stdout handling could retain unbounded output or deadlock above pipe capacity. The shared runner therefore writes stdin independently, drains both output pipes concurrently, caps allocations before reads, kills on overflow, and reaps before returning. Custom fsmonitor is inspected before `diff-files`, every Git false spelling is parsed explicitly, and transforming attributes are queried only for clean materialized tracked regular files that would otherwise substitute canonical index bytes.
