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

Audit finding `1806051` showed that treating every unsuccessful `rev-parse --is-inside-work-tree` result as affirmative non-Git detection permits corrupt, unreadable, misconfigured, ownership-rejected, or otherwise operationally broken repositories to fall through to filesystem evidence. Detection needs a narrow plain-directory classification and must preserve bounded Git diagnostics for every inconclusive failure.

The follow-up TOCTOU audit found that stable candidate capture alone is insufficient when project or scoped discovery runs before the capture retry. A governed tracked or untracked path can appear or disappear between inventory and capture without entering either candidate snapshot. Non-Git discovery similarly needs a post-payload inventory and a second payload/topology comparison, not merely two matching inventories before one read.

The subprocess cleanup audit found that byte caps do not bound a silent child and that early pipe/poll errors can return before termination, wait, and reader-thread cleanup. The bounded runner therefore needs a production wall-clock deadline plus ownership that cannot release a live or unreaped child on any ordinary error path; deterministic tests inject short deadlines and cleanup faults.

The pathspec audit found that placing raw governed names after `--` does not make them literal: Git still interprets magic prefixes and glob characters. Exact evidence commands need `:(top,literal)` encoding, and scoped discovery needs an explicit literal directory prefix plus recursion, so metacharacter and leading-colon names cannot hide dirtiness or expand evidence to unrelated files.

The exact-scope audit also found that a single unscoped `ls-files --stage -z` reads up to 256 MiB of unrelated index state before filtering. Stage/object metadata must instead be queried in literal exact-candidate batches with aggregate bounds, preserving relevant conflict rejection while making narrow evidence independent of unrelated repository scale.

Definition digest construction used `if let Ok(read_dir)` plus flattened and filtered entries, which made unreadable directories, individual entry failures, and non-UTF names indistinguishable from absent optional deltas. Definition inventory must fail closed and be generation-bound so terminal validation cannot authenticate an approval that silently omitted a racing or unreadable artifact.

Volatile filtering occurred after Git discovery bounds and after broad filesystem traversal, allowing huge or unreadable build, dependency, and active lifecycle trees to deny an otherwise narrow nonvolatile digest. Discovery must prune those results and subtrees during streaming, with a narrow exception only for exact paths the approved contract explicitly governs.
