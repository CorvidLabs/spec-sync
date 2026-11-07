# Lesson bundle: a-warm-status-of-an-archived-change-does-not-digest-the-whole-working-tree-to-report-its-review-currency

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: A warm status of an archived change does not digest the whole working tree to report its review currency
- **Kind**: Feature
- **Specs**: change
- **Paths**: src/change/historical_definition_digest_matches.rs, src/change_tests/fast_archived_reads.rs, specs/change, CHANGELOG.md
- **Acceptance**: warm_archived_status_git_processes_do_not_grow_with_tracked_files passes: 3000 more tracked files add fewer than 3 Git processes to a warm archived change status (committed failing first in a5b54557: 89 -> 196)
- **Acceptance**: an_archived_change_reports_the_scoped_review_currency_the_full_check_gives passes unchanged: an archived change reports the scoped_review_current the full check gives today, true right after finalize while the archive is uncommitted and false once committed, after later commits, after an unrelated edit and for a moved archive
- **Acceptance**: An archived change takes the project-input digest for scoped_review_current only while its archived package has uncommitted changes; a committed archive reports false without the digest
- **Acceptance**: States before archive compute scoped_review_current exactly as today; the next action, every gate and every refusal are unchanged
- **Acceptance**: On Raven (150 archives) a warm archived change status drops from 183 Git processes to about 53; before and after times are recorded

## Evidence

- Verification commit: `8749eec02de67074693a27240579e7063a5251f2`
- Base commit: `86857c8f63acc93b480ac441e32183b8b1ed3a29`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

After #979 fixed the cold path, the auditor's Raven dogfood (`_audits/dogfood-raven-latency-2026-11-06.md`) still put spec-sync's warm `change status` at 3.6 s or more on a loaded host, against SPEED-1.a's 2 s. The orc asked for a warm-path profile and fix, with REQ-change-147. SPEED-1.a covers warm reads, so there is no new HI.

On Raven (150 archives, 5,629 tracked files) a warm `change status <archived-id> --json` spawns 183 Git processes, about 8 s at load average 58. A symbolized macOS `sample` puts 62% of samples under `summarize_change_with_effective -> scoped_review_currency -> project_input_digest`. The summary computes `scoped_review_current` for every state, archived included. That takes the whole-tree project-input digest, two passes of 13 path chunks with 5 Git commands each: 130 of the 183 processes. The other 33% is `terminal_evidence_summary` answering cached archives.

## From the change's testing.md

# Testing

- `warm_archived_status_git_processes_do_not_grow_with_tracked_files` (new; fails today: 89 Git processes with no extra files, 196 with 3,000).
- `an_archived_change_reports_the_scoped_review_currency_the_full_check_gives` (new; passes today and must stay passing). Full check vs summary: finalized and uncommitted gives true; archive committed, after a later change, after an unrelated working-tree edit, and for a moved archive all give false.
- Unchanged: the warm and cold archived-read guards, `fast_finalize`, the lifecycle integration tests, and the verify lane.
- On demand: Raven warm `change status` processes and time, before and after.

## Results

| Guard | Before | After |
|---|---|---|
| `warm_archived_status_git_processes_do_not_grow_with_tracked_files` (0 vs 3000 extra files) | 89 vs 196, failed (`a5b54557`) | 70 vs 70 |
| `warm_archived_status_git_processes_do_not_grow_with_legacy_changes` (2 vs 8) | 91 vs 91 | 72 vs 72 |
| `an_archived_change_reports_the_scoped_review_currency_the_full_check_gives` | passes | passes (true right after finalize while uncommitted; false in the four committed cases) |
| `an_ignored_just_finalized_archive_reports_the_full_review_currency` (from review) | n/a | failed first (`f85d4a35`: summary false, full check true), passes |

Raven (150 archives, 5,629 tracked files; release builds; load average about 56), warm `change status <archived-id> --json`, three runs each:

| Build | Git processes | Wall |
|---|---|---|
| main after #979 | 183 | 6.2-13.1 s |
| This change | 65 | 1.8-1.9 s |

The JSON output is identical.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-147 | change tests `warm_archived_status_git_processes_do_not_grow_with_tracked_files` (failing first, `a5b54557`), `an_archived_change_reports_the_scoped_review_currency_the_full_check_gives` and `an_ignored_just_finalized_archive_reports_the_full_review_currency` (failing first, `f85d4a35`) |

## Where these lessons go

- `specs/change/change.3md`
