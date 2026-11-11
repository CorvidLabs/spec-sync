# Lesson bundle: re-approval-accepts-an-unchanged-delta-body-that-already-carries-the-moved-canonical-text

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Re-approval accepts an unchanged delta body that already carries the moved canonical text
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/collect_requirement_ids.rs, src/change_tests/stale_modified_delta.rs
- **Acceptance**: re-approving an unchanged MODIFIED or REMOVED delta whose canonical block moved since approval succeeds and records the new base when the approval-time base text, recovered from git by its recorded digest, shows in a line-level three-way comparison that the body already carries every edit the current text made; otherwise, including when the base cannot be recovered, it is refused as before and nothing is recorded; no new flags

## Evidence

- Verification commit: `39fb7f5d750b21cc0c9b686d837f04c32f4174ee`
- Base commit: `6ef61e6aae6d63b2586d0f9f216c47803dba4d9f`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #975 (found on chough#163). Change B's MODIFIED body was written on top of change A's
amended text while A was still open. B was approved while main had the pre-A text, so the
approval recorded the old base. When A landed, the block moved. Re-approving B was refused
("the delta body has not been rewritten") although B's body already carried A's text. Only a
byte-only edit recovered it.

Leif chose on 2026-11-10 (relayed by t370): recover the old base text from git and re-approve
only when the delta still applies cleanly to the new base, with no new flags.
`approval_delta_bases` now collects each unchanged, unapplied item whose base moved. For
each, it walks the canonical file's history (newest first, at most 200 commits) for the
version whose block digest equals the recorded base. It then runs a line-level diff3-style
comparison of base, current and approved body. When every region the current text changed is
either unchanged there or already identical in the body, the approval records the current
base. Otherwise the existing refusal stands, as it does outside git or when no commit matches.

## From the change's testing.md

# Testing

Tests in `src/change_tests/stale_modified_delta.rs`. The git-backed tests commit the
approval-time base, land the other change's edit, and re-approve. The positive test fails when
the three-way acceptance is disabled.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-122 | unit `three_way_keeps_body_accepts_only_bodies_that_carry_the_moved_edits`, `re_approving_an_unchanged_body_that_already_carries_the_moved_text_records_the_new_base`, `re_approving_an_unchanged_body_that_lacks_the_moved_text_is_refused_in_a_repository`, `re_approving_an_unchanged_stale_body_is_refused`, `re_approving_an_unchanged_current_body_keeps_its_base` |

## Where these lessons go

- `specs/change/change.3md`
