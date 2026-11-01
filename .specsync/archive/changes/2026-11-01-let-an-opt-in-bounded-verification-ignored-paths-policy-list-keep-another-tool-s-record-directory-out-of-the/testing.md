---
change: let-an-opt-in-bounded-verification-ignored-paths-policy-list-keep-another-tool-s-record-directory-out-of-the
artifact: testing
---

# Testing

Each new test carries an honest label:

- A DISCRIMINATOR fails on origin/main a80748ff and passes with the change. On a80748ff the key is
  ignored, so these fail on the assertion that verification or review stays current, or that an
  invalid list is refused.
- A CONTROL passes on both.

Labels are confirmed against a build of unchanged a80748ff from a separate checkout. Tests that
call new private functions cannot compile there. They are listed as such and left out of that run.

## Unit tests

These go in a new test part, `src/change_tests/verification_ignored_paths.rs`, included from
`src/change_tests.rs` and listed in the `change` spec's `files:`. The fixtures are Git
repositories built like those of `workspace_digest_tracks_unicode_and_space_paths`, with an
approved change for module `auth` covering `src/auth.rs`, approved by a fixture actor.

DISCRIMINATORS:

- `listed_prefix_untracked_file_keeps_verification_current`: verify, then add the untracked file
  `.chough/evidence/obs.json`. `recorded_verification_is_current` is true, and
  `summarize_change.next_action` names `change review`.
- `listed_prefix_tracked_edit_and_delete_keep_verification_current`: the same, for a tracked file
  that is edited, then deleted, then re-added and committed.
- `listed_prefix_uncommitted_edit_after_review_keeps_review_current`: record a review, then edit
  the tracked `.chough/obs.json` without committing. Scoped-review currency is `Current`, and
  acceptance succeeds.
- `invalid_verification_ignored_paths_fail_closed`, a table test. Each case makes
  `project_input_digest` and `validated_verification_ignored_paths` return `Err` containing
  `verification_ignored_paths`, the entry and the rule. The cases:
  - `*.json`, `.chough/**`, `[a]/`, `{a,b}/`, `!x/`;
  - `/abs/`, `C:/x/`, `a\\b/`, `../x/`, `./x/`, `a//b/`, `.`, `./`;
  - `.chough` (no slash), an empty string, a control character;
  - 129 bytes, 9 entries, a duplicate, and a nested pair `a/` with `a/b/`;
  - `.specsync/`, `.specsync/changes/`, `specs/`, `specs/auth/`, `src/`, `src/gen/`, `SRC/`;
  - a parent of a spec `files:` entry; `principles/` holding `principles_file`; a
    `custom_artifacts` template directory;
  - `tests/`, `tests/fixtures/`, `.github/` and `.github/workflows/` under the default
    `meaningful_paths`;
  - a symlinked `.chough` pointing at `src/`, and `.chough/` below a symlinked parent.
- `ignored_paths_never_unlocks_an_entry`: with `tests/fixtures/` and `.github/workflows/` also
  listed in `ignored_paths`, each is still refused as overlapping the `meaningful_paths` scope
  `tests/` or `.github/`. With `meaningful_paths: ["."]`, `.chough/` is refused whether or not
  `ignored_paths` lists it. This pins that `ignored_paths` keeps its one meaning.
- `invalid_list_is_refused_by_every_lifecycle_verb`: with the list `["src/"]`, each of
  `create_change`, `answer_question` (for `acceptance_criteria` and for `affected_paths`),
  `approve_definition`, `check_change`, review recording, `accept_change` and `audit_project`
  returns an error naming `src/`, `summarize_change.next_action` names fixing that entry, and no
  change directory, approval, verification attempt or review attempt is written.
- `entry_overlapping_declared_affected_paths_is_refused_for_that_change`: a change declaring
  `.chough/` or `.` is refused at `create_change`, at the `affected_paths` answer, at approval and
  at `check_change`. A second change that declares `src/auth.rs` is unaffected.
- `nested_project_root_excludes_only_its_own_prefix`: the project lives at `app/` inside the Git
  repository, with `.specsync/` under `app/` and the list `[".chough/"]`. An untracked
  `app/.chough/obs.json` keeps verification current. An untracked `app/notes/.chough/obs.json`
  stales it. The validator's symlink check follows `app/.chough`, not the repository-root
  `.chough`, and a symlinked `app/.chough` is refused.
- `summary_reports_the_active_list`: `summarize_change` carries
  `verification_ignored_paths: [".chough/"]`.
- `changing_the_list_after_verification_stales_it`: adding, editing or removing an entry (an
  `sdd.json` edit) makes verification and review stale.
- `verification_ignored_frame_cannot_collide_with_path_frames`: at the `FramedDigest` level,
  over one fixed set of entries, the digest with the list frame for `[".chough/"]` differs from
  the digest without it, from the digest with `[".chough/", "x/"]`, and from a digest without the
  frame whose entries add files at the paths `verification-ignored` and
  `verification-ignored-prefix` with content `.chough/` (both as `content` and as `object-id`
  frames). This is the counterpart of
  `framed_workspace_digest_resists_nul_entry_boundary_collisions` for the new frame.

CONTROLS:

- `empty_or_absent_list_digest_is_byte_identical`: for one fixture tree, the digest with no key,
  with `[]` and on a80748ff is the same pinned hex value. `verification.json` and
  `change status --json` bytes contain no `verification_ignored_paths`.
- `no_persisted_record_gains_a_field`: with the list `[".chough/"]`, the top-level keys of
  `verification.json`, `review.json` and the finalization record equal those written without the
  list, and an accepted-evidence anchor recorded with the list is found by
  `consider_accepted_evidence_anchor` exactly as one recorded without it.
- `unlisted_paths_still_stale_verification`: with `[".chough/"]` set, an untracked
  `notes/obs.json`, an edit to `src/auth.rs`, and a file named `.chough` each make verification
  stale.
- `listed_prefix_commit_after_review_still_stales_review`: record a review, then commit only
  `.chough/obs.json`. Scoped-review currency is `Stale`, with today's message from
  `review_commit_walk` naming `.chough/obs.json`.
- `unlisted_commit_after_review_still_stales_review`: a commit touching `notes/x` after the review
  is stale with today's message.
- `ignored_paths_and_coverage_are_unchanged_by_the_list`: `path_is_meaningful` answers for a table
  of paths are equal with and without the list.
- `write_default_policy_bytes_are_unchanged`: `write_default_policy` and adoption omit the key.
- `acceptance_input_digest_is_unchanged_by_the_list`: a change's acceptance manifest is identical
  with and without the list.
- Existing tests stay green unchanged:
  - `workspace_digest_tracks_unicode_and_space_paths`,
  - `framed_workspace_digest_resists_nul_entry_boundary_collisions`,
  - `workspace_digest_distinguishes_symlinks_files_and_targets`,
  - `git_lfs_clean_file_is_named_by_its_object_id`,
  - `non_git_walk_skips_volatile_trees_but_fails_on_relevant_walk_errors`,
  - `recorded_verification_is_stale_when_the_workspace_digest_does_not_match`,
  - `scoped_review_persistence_commit_keeps_verification_current`,
  - `scoped_review_persistence_mixed_with_source_change_is_stale`,
  - `default_policy_covers_root_action_and_dependency_lockfiles`,
  - `accepted_evidence_survives_squash_merge_from_nested_project_root`.

## Command-layer tests

In the existing test module of `src/commands/change.rs`, beside
`ship_status_product_stage_rejects_missing_verification`:

- `ship_status_blocks_on_an_invalid_verification_ignored_paths_entry` (DISCRIMINATOR): with the
  list `["src/"]` and a verified change, `ship_status_report` lists a blocker containing
  `verification_ignored_paths` and `src/`, and no blocker says the verification is stale.
- `status_and_check_text_render_the_list_only_when_non_empty` (DISCRIMINATOR for the non-empty
  case, CONTROL for the empty case).

## Integration tests

These go in `tests/integration/change.rs`, through the built binary. The step letters are those
of the issue #825 repro in `context.md`.

`listed_tool_records_do_not_stale_review_through_the_cli` (DISCRIMINATOR):

1. In a temp Git repository, `change adopt`, approve one change for `shipping` with a fixture
   actor, implement it and run `change check <id> --commit`. No list yet.
2. Write `"verification_ignored_paths": [".chough/"]` into `sdd.json` under the change's coverage,
   commit it, and run `change check <id> --commit` again (step D, first half).
3. Step B: add an untracked `.chough/evidence/obs.json`. `change status <id> --json` has
   `summary.next_action` naming `change review` and `summary.verification_ignored_paths` equal to
   `[".chough/"]`.
4. Step D, second half: remove the file, confirm status names review, restore it, and confirm
   status still names review.
5. Step F: commit `.chough/evidence/obs.json`. Status still names review.
6. `change review <id> --reviewer <fixture>` exits 0.
7. Edit `.chough/evidence/obs.json` without committing. `change finalize <id>` exits 0.

The text output of `change status` contains `Verification ignores: .chough/` and the standing
note.

`listed_prefix_commit_after_review_still_stales_through_the_cli` (CONTROL): steps 1 to 6, then
commit only an edit under `.chough/`. `change status` names `change review` again and
`change finalize` refuses with the descendant-walk message.

`invalid_verification_ignored_paths_are_refused_through_the_cli` (DISCRIMINATOR). With `["src/"]`,
`change new`, `change approve`, `change check` and `change review` each exit non-zero with
`invalid SDD policy: verification_ignored_paths`, `change status` names the entry as its next
action, and `change ship-status` lists the blocker.

`absent_list_keeps_issue_825_behaviour` (CONTROL). Steps 1, 3 and 5 without the key reproduce
today's refusal: `verification project-input digest is stale`.

## Regression lanes

- `cargo test change::`.
- `cargo test commands::change::`.
- `cargo test --test integration change::`.
- `fledge lanes run verify`.
- `specsync check --strict --require-coverage 100`.
- `specsync change audit`.
- `fledge lanes run pre-push` before every push.
- `fledge trust verify` before calling the work complete.

## Requirement evidence

| Requirement | Required evidence |
|-------------|-------------------|
| REQ-change-114 | Unit DISCRIMINATORS: `listed_prefix_untracked_file_keeps_verification_current`, `listed_prefix_tracked_edit_and_delete_keep_verification_current`, `listed_prefix_uncommitted_edit_after_review_keeps_review_current`, `changing_the_list_after_verification_stales_it`, `nested_project_root_excludes_only_its_own_prefix`, `verification_ignored_frame_cannot_collide_with_path_frames`. CONTROLS: `empty_or_absent_list_digest_is_byte_identical`, `no_persisted_record_gains_a_field`, `unlisted_paths_still_stale_verification`, `listed_prefix_commit_after_review_still_stales_review`, `unlisted_commit_after_review_still_stales_review`, `ignored_paths_and_coverage_are_unchanged_by_the_list`, `acceptance_input_digest_is_unchanged_by_the_list`, `write_default_policy_bytes_are_unchanged`. Integration: `listed_tool_records_do_not_stale_review_through_the_cli`, `listed_prefix_commit_after_review_still_stales_through_the_cli`, `absent_list_keeps_issue_825_behaviour`. |
| REQ-change-115 | `invalid_verification_ignored_paths_fail_closed`, `ignored_paths_never_unlocks_an_entry`, `invalid_list_is_refused_by_every_lifecycle_verb`, `entry_overlapping_declared_affected_paths_is_refused_for_that_change`, `nested_project_root_excludes_only_its_own_prefix`, `invalid_verification_ignored_paths_are_refused_through_the_cli`. |
| REQ-cmd-change-024 | `summary_reports_the_active_list`, `ship_status_blocks_on_an_invalid_verification_ignored_paths_entry`, `status_and_check_text_render_the_list_only_when_non_empty`, and the text assertions in `listed_tool_records_do_not_stale_review_through_the_cli`. |

## Observed results

Labels were confirmed against an unmodified export of a80748ff (`git archive` of the approved
commit, whose `src/` and `tests/` are identical to a80748ff) with the final test files copied in.
The two NEW API tests (`verification_ignored_frame_cannot_collide_with_path_frames`,
`status_and_check_text_render_the_list_only_when_non_empty`) cannot compile there and were left
out of that run.

- Unit, on a80748ff: 11 failed and 9 passed. The 11 failures are exactly the DISCRIMINATORS:
  `listed_prefix_untracked_file_keeps_verification_current`,
  `listed_prefix_tracked_edit_and_delete_keep_verification_current`,
  `listed_prefix_uncommitted_edit_after_review_keeps_review_current` (review refused with
  `verification project-input digest is stale`), `invalid_verification_ignored_paths_fail_closed`
  (`["*.json"] must be refused`), `ignored_paths_never_unlocks_an_entry`,
  `invalid_list_is_refused_by_every_lifecycle_verb` (`change new must refuse`),
  `entry_overlapping_declared_affected_paths_is_refused_for_that_change`,
  `entry_overlapping_declared_affected_paths_is_refused_at_approve_and_check`,
  `nested_project_root_excludes_only_its_own_prefix`, `summary_reports_the_active_list` (`Null`)
  and `ship_status_blocks_on_an_invalid_verification_ignored_paths_entry` (blockers said
  "verification evidence is stale" and "scoped review is stale"). The 9 passes are the CONTROLS.
- Label corrections against the draft: `changing_the_list_after_verification_stales_it` is a
  CONTROL, because any `sdd.json` edit already stales verification on a80748ff. The added
  `entry_overlapping_declared_affected_paths_is_refused_at_approve_and_check` covers the approve
  and check half of that table row.
- Integration, on a80748ff: `listed_tool_records_do_not_stale_review_through_the_cli` fails at
  step B (`next_action` is `run specsync change check ...`), and
  `invalid_verification_ignored_paths_are_refused_through_the_cli` fails because `change new`
  succeeds. `listed_prefix_commit_after_review_still_stales_through_the_cli` and
  `absent_list_keeps_issue_825_behaviour` pass.
- With the change: all 22 unit and command-layer tests and all 4 integration tests pass. The full
  suite passes: `cargo test` reports 2645 unit tests and 468 integration tests, 0 failed.
  `cargo clippy -- -D warnings` and `cargo fmt --check` are clean.

### Issue #825 replay with the built binary

The issue's `base.sh` was run unchanged against Chough's `examples/shipping` at 2756dc2, with a
release build of this branch first on `PATH`. A replay script then listed `.chough/`, re-verified,
and walked steps B, D and F, review and finalize. Actors are `SIMULATED-...` in a throwaway
repository.

```text
specsync 6.1.1
== A. verified, clean tree
next_action: run `specsync change review free-shipping-at-the-threshold --reviewer <human>` after the PR's scoped review passes
== D1. list .chough/ in verification_ignored_paths, commit, re-verify
next_action after the sdd.json commit: run `specsync change check free-shipping-at-the-threshold`
Verified, committed, and consistent with the committed tree.
  Verification ignores: .chough/
    note: SpecSync cannot tell whether project tests or CI read files under these prefixes; edits there do not stale verification.
next_action after re-verify: run `specsync change review free-shipping-at-the-threshold --reviewer <human>` after the PR's scoped review passes
summary.verification_ignored_paths: [".chough/"]
== B. one untracked file under .chough/
next_action: run `specsync change review free-shipping-at-the-threshold --reviewer <human>` after the PR's scoped review passes
== D2. remove the record, then restore it
next_action without it: run `specsync change review free-shipping-at-the-threshold --reviewer <human>` after the PR's scoped review passes
next_action restored: run `specsync change review free-shipping-at-the-threshold --reviewer <human>` after the PR's scoped review passes
== F. commit the record before the review
next_action after committing .chough/: run `specsync change review free-shipping-at-the-threshold --reviewer <human>` after the PR's scoped review passes
  Verification ignores: .chough/
    note: SpecSync cannot tell whether project tests or CI read files under these prefixes; edits there do not stale verification.
== review
review exit=0 
== uncommitted edit under .chough/ after the review, then finalize
next_action: run `specsync change finalize free-shipping-at-the-threshold`
finalize exit=0
```

## Review round 1 (#830)

Independent reviewers found one fail-open hole and three gaps against this plan. Each was
re-verified before it was fixed.

- Deviation from the enumerated REQ-change-115 criteria, for the scope owner to confirm.
  `change check` also reads, in-process, the configured `schema_dir` (replayed by
  `get_schema_table_names` and `build_schema_columns` in `evaluate_spec_code_sync` and
  `validate_effective_contracts` to judge spec `db_tables`), every path-form spec `depends_on`
  entry (an existence check that errors) and every file a spec's `Consumed By` table names (an
  existence check that warns, so it can fail `--strict`). None was in the protected set, so an
  entry such as `db/schema/` was accepted, a commit dropping a table left verification and review
  current, and a fresh `change check` failed. All three are now refused, under the REQ's own words
  "overlaps no specs, sources, SDD policy inputs". The scope owner (0xLeif) chose to amend the
  enumerated acceptance criteria, REQ-change-115 and Contract item 24 to name all three, so the
  definition now matches the refusal. Root-level inputs
  (`.specsyncignore`, legacy config files) need no entry, because a directory prefix cannot cover
  a root file.
- New DISCRIMINATORS, which fail on a80748ff and on the pre-fix head `de3e5ae2`:
  `entry_covering_an_in_process_check_input_is_refused` (`["db/"] must be refused`) and, in
  `tests/integration/change.rs`, `entry_covering_schema_dir_is_refused_through_the_cli`
  (`change new` exits 0 and prints `Verification ignores: db/schema/`).
- `listed_prefix_commit_after_review_still_stales_review` now matches `Stale(message)` and asserts
  that the message is the descendant walk's own, naming `.chough/obs.json`. It is a DISCRIMINATOR
  against a80748ff, where the review goes stale through the digest first (`the reviewed project
  files changed after the review was recorded`). Its integration twin no longer guards its
  assertion: it asserts that status names `change review` and that `change ship-status --json`
  gives the walk's message as a blocker reason. On a80748ff it fails with `next_action` =
  ``run `specsync change check ship-the-auth-module` ``. The earlier report's claim that the twin
  asserted the walk message was wrong. It asserted only the generic finalize refusal, and only
  inside a guard.
- `no_persisted_record_gains_a_field` left two plan items unexercised. Both now have their own
  CONTROLS. `accepted_evidence_anchor_is_found_identically_with_the_list` squash-merges an
  accepted change with and without the list. It then authenticates it through
  `authenticated_accepted_transition`, which is `consider_accepted_evidence_anchor`, along with
  `ensure_closing_approval_valid` and the history lookup. `archived_change_is_unaffected_by_a_later_list`
  finalizes, then adds and edits the list in the working tree. The full archive-integrity walk
  (`check_project`) reports the same terminal evidence, and `change audit` reports exactly what it
  reports for a non-list policy edit. Committing a policy edit with no change covering it trips
  the existing policy-change guard, whatever the edit is.
- `empty_or_absent_list_git_digest_is_byte_identical` (CONTROL) pins the Git-discovery digest
  for a committed tree, with fixed modes and one untracked file, with the key absent
  (`1300fd3d...5da8`) and as `[]` (`7750e475...dbec6`). Both values were computed on a80748ff.
  `empty_list_adds_no_key_to_verification_or_status_json` (CONTROL) writes `[]` explicitly in a
  Git project and finds no key in verification.json or in the serialized `ChangeSummary`.

