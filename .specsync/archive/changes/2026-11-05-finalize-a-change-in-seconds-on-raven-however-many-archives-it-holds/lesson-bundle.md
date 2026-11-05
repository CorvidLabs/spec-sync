# Lesson bundle: finalize-a-change-in-seconds-on-raven-however-many-archives-it-holds

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Finalize a change in seconds on Raven however many archives it holds
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/read_cache.rs, src/change/validate_loaded_change.rs, src/change/historical_definition_digest_matches.rs, src/change/consider_accepted_evidence_anchor.rs, src/change_tests/fast_finalize.rs, src/change_tests.rs, specs/change
- **Acceptance**: SpecSync archive (finalize) no longer takes minutes on Raven: the Git processes one finalize spawns grow by a small bounded number per other archived change instead of re-walking every archive's history, and every verdict and refusal is identical to a live run.

## Evidence

- Verification commit: `b8dfab33b12c9baa109fd1b22c743d9112a33e0e`
- Base commit: `df7bace2742ff5910d9b6747ecfda7e639848c7e`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Leif asked in Grok Bot chat t813u (2026-11-03) for SpecSync changes and archive and everything to work fast, after reports that "sometimes SpecSync archive takes like three minutes to run" (hi SPEED-2). This stacks on REQ-change-119 (#840), which made archived reads fast but left mutating commands reading live.

Profiled on Linux against a scratch branch of CorvidLabs/raven (127 archived changes): one `change finalize` with the #840 build took 50.1 s quiet and 101 s under load, spawning 10,948 Git processes and spending 53.6 s in Git. Almost all of it was the archive walk finalize runs to prove the new archive fits: every other archive's workflow-version history and workflow-v2 baseline history re-derived from scratch, plus 2,170 `rev-parse --show-prefix` calls.

## From the change's design.md

# Design

1. A `HistoryCacheScope` that `finalize_change` and `archive_change` open for their own duration. It shares the persistent, Git-keyed layer of REQ-change-119 (workflow-version history verdicts and archive introductions, under the same digest and history rule) and flushes it on drop. It takes no working-tree snapshot, so the command still sees its own writes.
2. Inside that scope, a memo for Git queries whose answer is a pure function of objects named by full id: the bytes of a path at a full commit id, `rev-list --parents -n 1 <id>`, `merge-base --is-ancestor <id> <id>`, plus the repository prefix and a positive repository detection. Queries naming `HEAD`, a branch, or the working tree are never memoized; failed process runs are not stored.
3. The record-independent half of the workflow-v2 baseline check moves into `validate_workflow_v2_baseline_history`, which names the resolved `HEAD` id in every query instead of `HEAD`. A passing verdict is memoized in the scope keyed by that id, the baseline path, the cutoff and the current baseline bytes; `HEAD` itself is still resolved live on every call, and refusals are always re-derived.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-121 | `finalize_git_processes_do_not_grow_with_other_archives` failed first on #840 (four more archives added 654 Git processes to one finalize, 1,296 with few and 1,950 with four more) and passes with at most 12 per archive. The full change suite, including every finalize and archive refusal test, passes unchanged. |

## Where these lessons go

- `specs/change/change.3md`
