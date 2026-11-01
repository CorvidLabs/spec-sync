# Lesson bundle: let-an-opt-in-bounded-verification-ignored-paths-policy-list-keep-another-tool-s-record-directory-out-of-the

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Let an opt-in bounded verification_ignored_paths policy list keep another tool's record directory out of the verification project-input digest
- **Kind**: Feature
- **Specs**: change, cmd_change
- **Paths**: src/change.rs, src/change/verification_ignored_paths.rs, src/change/sdd_version.rs, src/change/collect_requirement_ids.rs, src/change/validate_loaded_change.rs, src/change/historical_definition_digest_matches.rs, src/change_tests.rs, src/change_tests/verification_ignored_paths.rs, src/commands/change.rs, tests/integration/change.rs, site/src/content/docs/configuration.md, site/src/content/docs/workflow.md, specs/change, specs/cmd_change
- **Acceptance**: With `.specsync/sdd.json` carrying `"verification_ignored_paths": [".chough/"]` committed and the change re-verified, adding, editing or deleting a file under `.chough/`, tracked or untracked, and committing such files before the review, leaves the project-input digest unchanged: `change status <id> --json` keeps `summary.next_action` at `specsync change review`, `change review` records a passing review, and `change finalize` accepts it. On the built binary this is the issue #825 repro with the list in place of the workarounds it tried: step B (an untracked `.chough/evidence/obs.json` after verification), step D (the list committed, the change re-verified, then the record restored) and step F (the record committed before the review) each leave status naming `change review`.
- **Acceptance**: The same additions, edits, deletions and commits outside every listed prefix (for example `notes/obs.json` or `src/lib.rs`) still stale verification and review exactly as today, and a path equal to a listed prefix without its trailing slash (a file or symlink named `.chough`) is not excluded. The scoped-review descendant walk is unchanged: any commit after the review, including one that touches only paths under a listed prefix, stales the review as today. An uncommitted edit under a listed prefix after the review leaves the review current, because the review binds the same project-input digest.
- **Acceptance**: An entry is refused with a message naming the entry and the rule it breaks unless it is a portable relative directory prefix ending in `/`: no glob or pattern characters, no absolute path, no empty, `.` or `..` component, no backslash or control character, no symlink at or above the entry on disk, at most 128 bytes, no duplicate or nested entries, and at most 8 entries.
- **Acceptance**: An entry is refused when it equals, contains or lies inside `.specsync/`, the configured specs tree, a configured `source_dirs` entry, the configured `schema_dir`, any `meaningful_paths` scope, an SDD policy file, the configured `principles_file` or a `custom_artifacts` template, or when it contains a file listed in any spec's `files:`, a path-form `depends_on` entry of any spec (one naming a path, not a bare module name or a cross-project reference) or a file named in any spec's `Consumed By` table. `ignored_paths` does not relax this: an entry overlapping a `meaningful_paths` scope, such as `tests/fixtures/` or `.github/workflows/` under the default policy, is refused even when an `ignored_paths` scope also covers it. Overlap is compared ignoring ASCII case. Entries are project-relative, so a project rooted in a subdirectory of its Git repository excludes only paths under `<project>/<entry>`. The list never changes which paths are meaningful.
- **Acceptance**: An entry that overlaps a declared `affected_paths` scope of a change is refused for that change at `change new`, at `change answer <id> affected_paths`, and at `change approve`, `change check`, `change review` and `change finalize`, naming the entry and the change's path.
- **Acceptance**: An invalid list fails closed in every lifecycle verb that loads SDD policy, naming the entry and the rule it breaks: `change new`, `change answer`, `change approve`, `change check`, `change review`, `change finalize`, `change ship` and `change audit` refuse; `change status` makes fixing that entry its one next action; and `change ship-status` reports it as a blocker naming the entry rather than as stale verification. No evidence is recorded against an invalid list. `specsync check` does not read the list.
- **Acceptance**: Adding, removing or editing an entry after verification stales verification and review, because `.specsync/sdd.json` is a project input. No persisted record gains a field: `verification.json`, `review.json` and finalization evidence bind the list only through the bytes of `.specsync/sdd.json` and the list frame in the project-input digest. When the list is non-empty, `change status --json` reports it as `summary.verification_ignored_paths`, and `change check` and `change status` text print it with a standing note that SpecSync cannot tell whether project tests or CI read files under those prefixes.
- **Acceptance**: With the list absent or empty, the project-input digest, `verification.json`, `review.json`, status JSON and every other persisted byte are identical to origin/main a80748ff, and `ignored_paths`, `meaningful_paths`, `config.toml` `exclude_dirs` and `exclude_patterns`, coverage, acceptance-input digests, the scoped-review descendant walk and `change check --commit` staging are unchanged.
- **Acceptance**: Regression tests are labelled DISCRIMINATOR (fail on origin/main a80748ff) or CONTROL (pass on both), and include an integration test through the built binary that, with the list set, replays issue #825 steps B, D and F as named in the first criterion, records a passing review and finalizes, and a second that shows a commit after the review touching only the listed prefix still stales the review.

## Evidence

- Verification commit: `b36cfa825652e60a0d8ea0b38f1459cbdb75a56e`
- Base commit: `a80748ff44ff42cd7578ac0f7cb8ba7e080ab183`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

Issue #825, reported downstream as CorvidLabs/chough#12 (found during CorvidLabs/raven#644).

A workflow tool next to SpecSync, here Chough, writes its records under its own directory
(`.chough/`). It writes them between `change check` and `change review`, because that is when the
human review is prepared. Any such file makes the recorded verification stale, and
`change review` refuses with `verification project-input digest is stale`. No SpecSync setting
excludes the directory:

- SDD `ignored_paths` has no effect on the digest.
- `config.toml` `exclude_dirs` and `exclude_patterns` have no effect on the digest.
- Git ignore rules hide the files only while they are untracked. The records are meant to be
  committed and shared.

The options left to the user are all bad: stash the records before every review, keep them
untracked, or re-run `change check` after every write.

## Reproduction on current main

The issue's `base.sh` and `repro1.sh` were run unchanged, except that the holding directory moved
from `/tmp` to a disposable scratch directory. The fixture was Chough's `examples/shipping` at
CorvidLabs/chough@2756dc2, with approvals and reviews by actors labelled `SIMULATED-...` in a
throwaway repository. No human decision is recorded there.

The binary was built from origin/main a80748ff (`cargo build --release`; it reports
`specsync 6.1.1`). For this revision the repro was run again, twice in parallel in fresh
directories with the fixture copied from a local CorvidLabs/chough checkout at 2756dc2: once with
that build and once with the installed `specsync 6.1.1`. Both logs were kept and compared with
`diff`. The only differing lines were the two commit ids in step G (`7edcce0`/`e20ac41` for the
installed binary, `7884e1f`/`d897cd2` for the a80748ff build). The output of the a80748ff build:

```text
== A. verified, clean tree
next_action: run `specsync change review free-shipping-at-the-threshold --reviewer <human>` after the PR's scoped review passes
== B. one untracked file under .chough/ (another tool's record)
next_action: run `specsync change check free-shipping-at-the-threshold`
exit=1 error: scoped review cannot bind stale verification (verification project-input digest is stale); run `specsync change check` first
== C. same file hidden through .git/info/exclude
next_action: run `specsync change review free-shipping-at-the-threshold --reviewer <human>` after the PR's scoped review passes
== D. add .chough/ to sdd.json ignored_paths, commit, re-verify, then restore the record
Committed the verification evidence.
Verified, committed, and consistent with the committed tree.
next_action after re-verify (no .chough/ yet): run `specsync change review free-shipping-at-the-threshold --reviewer <human>` after the PR's scoped review passes
next_action with .chough/evidence/obs.json present: run `specsync change check free-shipping-at-the-threshold`
exit=1 error: scoped review cannot bind stale verification (verification project-input digest is stale); run `specsync change check` first
== E. config.toml exclude_dirs = [".chough"] also does not help
3:exclude_dirs = [".chough"]
4:exclude_patterns = [".chough/**"]
Verified, committed, and consistent with the committed tree.
next_action with .chough/evidence/obs.json present: run `specsync change check free-shipping-at-the-threshold`
== F. commit the record instead
next_action after committing .chough/: run `specsync change check free-shipping-at-the-threshold`
exit=1 error: scoped review cannot bind stale verification (verification project-input digest is stale); run `specsync change check` first
== G. a tracked edit under .chough/ during change check --commit
check exit=0
7884e1f chore(lifecycle): record free-shipping-at-the-threshold verification
d897cd2 chore(lifecycle): materialize free-shipping-at-the-threshold
.chough/evidence/obs.json
```

One more probe ran in the same repository. With an unknown `"verification_ignored_paths": [".chough/"]`
key added to `sdd.json`, `change status` still parsed the policy and reported
`run specsync change check ...`. Today's binaries therefore ignore the key silently, which matters
for compatibility (see `design.md`).

## Cause

1. `project_input_digest_uncached` (`src/change/collect_requirement_ids.rs:3326`) hashes every
   discovered project path except those for which `project_input_is_volatile` (`:3777`) is true.
   That predicate is a fixed, built-in list: `.git/`, `target/`, `node_modules/`, the site build
   outputs, `.specsync/changes/`, `.specsync/archive/`, `.specsync/hashes.json`, the lock and the
   transaction journal. No policy input reaches it.
2. Discovery is `git ls-files --cached --others --exclude-standard` in `git_project_paths`
   (`:3356`), filtered by the matching hard-coded byte predicate `git_path_bytes_are_volatile`
   (`:3453`). Untracked ignored files are left out, which is why step C helps. Tracked files are
   listed whatever the ignore rules say, which is why steps F and G do not help. Outside Git,
   `strict_walk_project_paths` (`:3501`) applies the same fixed predicate.
3. `SddPolicy` (`src/change/sdd_version.rs:672`) has no field for this. `ignored_paths` is read
   only by the coverage predicate `path_is_meaningful_with_specs`
   (`src/change/validate_loaded_change.rs:1113`) and by the coverage error note
   (`uncovered_paths_error`, `:829`). Nothing under `src/change/` reads `config.toml`
   `exclude_dirs` or `exclude_patterns`. They belong to spec validation (`src/config.rs`,
   `src/validator/`), which explains steps D and E.
4. One digest is compared everywhere, so a single stray file stales every gate:
   - verification records it (`verify_change_locked`, `src/change/sdd_version.rs:3100`);
   - status and review compare against it (`verification_is_current_checked`,
     `src/change/validate_loaded_change.rs:3424`, and scoped-review currency,
     `src/change/historical_definition_digest_matches.rs:1808`);
   - acceptance compares against it (`:2058`), and finalization requires the review, verification
     and finalization digests to be equal (`:1621`, `:1673`);
   - `change audit` compares against it (`:4057`).
5. There is a second binding that the digest does not cover. After a review, the descendant walk
   `review_commit_walk` (`src/change/historical_definition_digest_matches.rs:1953`) stales the
   review for any commit that touches a path outside this change's own lifecycle records
   (`:2019`-`:2034`), using repository-relative paths (`git_repo_relative_path`, `:1972`). None of
   the repro steps commits after the review, so this draft leaves the walk unchanged: a Chough
   record committed after the review still stales it, as the AGENTS.md "review and ship are one
   step" rule already says. Relaxing it is a possible follow-up (`tasks.md`).
6. Step G is a separate behaviour: `change check --commit` (`run_checked_commit`,
   `src/commands/change.rs:4119`) stages tracked edits, so a tracked `.chough/` edit lands in the
   `chore(lifecycle): materialize ...` commit. REQ-change-102 makes the change domain the only
   owner of that staging decision.

## Lessons and constraints that apply

- The REQ-change-103 lesson in `specs/change/change.3md`: the digest is
  `specsync.project-input-digest.v4`, it names clean tracked files by object id, and unsupported
  filters fail closed. An empty list must leave it byte-identical.
- Issue #676: an `ignored_paths` override that was disclosed only at audit, "three verbs too
  late". An invalid entry in the new list has to be refused when the policy is loaded, by every
  lifecycle verb.
- `ignored_paths` means "not meaningful for SDD coverage". The issue proposes a separate key
  because #676 shows how easily two meanings get mixed. This draft keeps them separate.
- REQ-change-109: no change source file may exceed 5,000 lines. The five `src/change/*.rs` parts
  are at 4,150-4,434 lines, so the new policy code goes into a new include part.

## Human intent

This draft links existing criteria and captures none:

- SHIP-2.b: editing the code after a review makes that review stale. Committing anything after a
  review still stales it, because the descendant walk is unchanged. One narrowing remains and is
  open for the human (open question 2 in `design.md`): an *uncommitted* edit under a listed prefix
  after the review leaves the review current. Entries cannot overlap sources, specs or meaningful
  paths, so this draft reads such records as not "the code", but that reading is not settled.
- SHIP-3.a: finalizing needs verification and review that are both still current.
- SHIP-1.d: the evidence matches the tree it describes, except for the prefixes the human named.
- CHANGE-7: one next action. An invalid list becomes the one next action in `change status`.

Possible wording for a new criterion, for the human to accept, reword or reject. It is not
captured, and no `hi` command has been run:

> Records another tool keeps beside my project, in a folder I named, do not make my change's
> verification go stale before it is reviewed.

If the human decides that SHIP-2.b should not be narrowed for uncommitted edits either, SHIP-2.b
needs no change and the design must bind the review to an unfiltered digest instead. If the human
accepts the narrowing, SHIP-2.b's wording may need a human-approved adjustment, for example to say
"the code or anything SpecSync verifies". Neither is proposed as captured text here.

## From the change's design.md

# Design

## Policy shape

`.specsync/sdd.json` gains one optional key:

```json
{
  "verification_ignored_paths": [".chough/"]
}
```

- `SddPolicy` (`src/change/sdd_version.rs:672`) gains `pub verification_ignored_paths: Vec<String>`.
  Its default is empty, and it is serialized only when non-empty, so `write_default_policy` and
  `change adopt` keep writing today's bytes.
- `.specsync/sdd.json` stays a protected meaningful file, so adding, removing or editing an entry
  needs a covering change, as today.
- Entries are **project-relative**, like every other path in `sdd.json`.

## One validator, called by every lifecycle verb that loads policy

`src/change/verification_ignored_paths.rs` is a new include part, wired from `src/change.rs`,
which keeps every part under the 5,000-line limit. It holds:

- `validated_verification_ignored_paths(root) -> Result<Vec<String>, String>`. It loads the policy
  with `load_policy_checked` (a missing policy is an empty list; an unparseable one is today's
  invalid-policy error), then checks the entry rules in `research.md` against the live
  `config.toml` (`specs_dir`, `source_dirs`, `schema_dir`), the policy itself (`meaningful_paths`,
  `principles_file`, `custom_artifacts`), every spec's `files:`, path-form `depends_on` entries
  and `Consumed By` file references, and the on-disk symlink state. It
  returns the sorted entries, and it is memoized in the same read scope as `project_input_digest`
  so status and audit do not repeat it per change. The error names the first offending entry and
  the rule, for example
  ``invalid SDD policy: verification_ignored_paths entry `src/gen/` overlaps source_dirs `src` ``.
  It never reads `ignored_paths`: an entry that overlaps a `meaningful_paths` scope is refused
  whatever `ignored_paths` says, so `ignored_paths` keeps its one meaning and cannot unlock
  digest exclusion.
- `change_conflicts_with_verification_ignored_paths(record, entries) -> Result<(), String>`. It
  refuses any entry that overlaps one of the record's `affected_paths`, and names both.
- `path_is_verification_ignored(path, entries) -> bool`. A pure byte-prefix test on a
  project-relative path: `path` starts with an entry, which always ends in `/`. A path equal to the
  entry without its slash does not match.

Following the #676 lesson, the list is refused at load, not at audit. The call sites, each
refusing before it records anything:

| Verb | Where the validator runs | What the user sees on an invalid list |
|------|--------------------------|----------------------------------------|
| `change new` | `create_change` (`src/change/sdd_version.rs:1592`), full validator, then the per-change overlap check | refusal naming the entry; no workspace is created |
| `change answer` | `answer_question_with_snapshot` (`:2462`), full validator for every question, plus the overlap check after an `affected_paths` answer | refusal naming the entry; the record is not saved |
| `change approve` | `approve_definition_with_projection` (`:2762`), full validator, then the overlap check | refusal; no approval is appended |
| `change check` | `verify_change_locked` (`:3100`), through `project_input_digest`, plus the overlap check | refusal; no verification attempt is appended |
| `change review`, `change finalize`, `change ship` | through `project_input_digest` (`historical_definition_digest_matches.rs:1808`, `:2058`), plus the overlap check | refusal naming the entry |
| `change audit` | through `project_input_digest` (`:4057`) | audit failure naming the entry |
| `change status` | `summarize_change_with_effective` next action (`historical_definition_digest_matches.rs:3487`), right after the sequence-ledger freeze and before every state-local action | the one next action is ``fix `.specsync/sdd.json` verification_ignored_paths: <error>`` |
| `change ship-status` | `ship_status_report` (`src/commands/change.rs:866`), before the verification-currency check | a blocker naming the entry and the rule |

`change status` and `change ship-status` need their own call because they read verification
currency through the boolean helpers `verification_is_current` and
`recorded_verification_is_current` (`src/change/validate_loaded_change.rs:3394`, `:3413`), which
turn a digest error into `false`. Through the digest alone, an invalid list would show as "stale,
run `change check`". The explicit call makes both commands name the entry. The boolean helpers are
not changed. `specsync check` never loads the list.

## The digest

`project_input_digest_uncached` (`src/change/collect_requirement_ids.rs:3326`):

1. Run `validated_verification_ignored_paths`. An error is returned as the digest error, so every
   caller that propagates it refuses.
2. With an empty list, the code path is exactly today's: same discovery, same frames, same
   `specsync.project-input-digest.v4` domain. The digest is byte-identical.
3. With a non-empty list:
   - Discovery leaves out every path under an entry before capture, in both `git_project_paths`
     (`:3356`) and `strict_walk_project_paths` (`:3501`). Excluded files are never read, never
     count toward the payload bound, and a tool writing there during inspection cannot trip the
     "changed during inspection" retry.
   - Right after the `domain` frame, and before the first entry, the digest adds a list frame,
     then one frame per sorted entry:

     ```text
     frame(b"verification-ignored", <entry count as u64 big-endian>)
     frame(b"verification-ignored-prefix", <entry bytes>)   // once per entry, sorted
     ```

     `FramedDigest::frame` (`src/change/sdd_version.rs:397`) length-prefixes both the tag and the
     value. Every entry today starts with an `entry` tag and carries only `path`, `kind`, `mode`,
     `content` or `object-id` tags, and paths and file bytes appear only as *values* of those
     frames. So no path or file bytes can produce a `verification-ignored` or
     `verification-ignored-prefix` tag, in the same way the `object-id` frame is a frame that file
     bytes cannot match (REQ-change-103). The bytes of `sdd.json` already bind the list; the frame
     makes the binding explicit and keeps a digest computed with a list distinct from one computed
     without it.

### Project-relative and repository-relative paths

The project root can sit in a subdirectory of its Git repository. The entries and the digest's
discovery use the same frame:

- `git_project_paths` runs `git ls-files` with the project root as its working directory
  (`rooted_git_command`, `src/change/consider_accepted_evidence_anchor.rs:318`, sets
  `current_dir(root)`), and without `--full-name`, so Git prints paths relative to the project root. Those are the paths
  `path_is_verification_ignored` tests, so no conversion is needed and `app/.chough/` is never
  confused with `.chough/` at the repository root.
- The filesystem walk is rooted at the project root and produces project-relative paths.
- The symlink check joins each entry to the project root (`root.join(entry)`) and walks its
  components up to the root with `symlink_metadata`.
- Nothing in this change reads repository-relative paths. The descendant walk, which does
  (`git diff-tree` output converted with `git_repo_relative_path`,
  `src/change/historical_definition_digest_matches.rs:1972`), is unchanged. A follow-up that
  touched the walk would have to convert entries the same way.

A test with the project in a repository subdirectory (`testing.md`) pins this.

## The review descendant walk is not changed

`review_commit_walk` (`src/change/historical_definition_digest_matches.rs:1953`) keeps today's rule:
after a review, any descendant commit that touches a path outside this change's own lifecycle
records stales the review. A commit that touches only `.chough/` after the review therefore
stales it, exactly as on a80748ff and as the AGENTS.md rule "do not commit between review and
ship" already says.

An earlier revision of this draft relaxed the walk for listed prefixes. That was dropped:

- the issue asks only for the digest exclusion;
- the walk is a separate review-currency gate, and relaxing it narrows HI SHIP-2.b;
- it would have to bind the list in the policy blob at `review.implementation_commit`, not the
  working tree's, to be fail-closed on its own.

Relaxing it can be proposed as a follow-up if the human wants it (see `tasks.md`).

## Disclosure

- **No persisted record gains a field.** `VerificationRecord` bytes are compared byte for byte by
  accepted-evidence anchoring (`consider_accepted_evidence_anchor.rs:2132`-`:2153`) and feed
  `accepted_evidence_key`, and `ScopedReviewRecord` and `FinalizationRecord` are digest preimages.
  Every one of them already binds the list through `workspace_digest`, which covers the
  `sdd.json` bytes and the list frame. Keeping the list out of `verification.json` also means an
  older binary that re-serializes the record cannot drop a field.
- `ChangeSummary` gains `verification_ignored_paths`, omitted when empty, so
  `change status --json` exposes it as `summary.verification_ignored_paths`. `ChangeSummary` is
  output only and is never persisted.
- Text output from `change status` and `change check` adds, only while the list is non-empty:

  ```text
  Verification ignores: .chough/
    note: SpecSync cannot tell whether project tests or CI read files under these prefixes; edits there do not stale verification.
  ```

## Resulting behaviour

| Situation | origin/main a80748ff | After, list `[".chough/"]` | After, list absent or empty |
|-----------|----------------------|----------------------------|-----------------------------|
| Untracked file added under `.chough/` after check (step B) | stale | current | stale (unchanged) |
| List committed, re-verified, record restored (step D, with the list instead of `ignored_paths`) | not applicable | current | not applicable |
| Tracked `.chough/` file edited or deleted | stale | current | stale |
| `.chough/` committed between check and review (step F) | stale | current; the review records the new `HEAD` | stale |
| Uncommitted `.chough/` edit after review | review stale | review current | review stale |
| `.chough/`-only commit after review | review stale (walk) | review stale (walk, unchanged) | review stale |
| Edit under `src/` or `notes/` | stale | stale | stale |
| A file or symlink named `.chough` (no slash) | stale | stale | stale |
| Entry added to, edited in or removed from the list | not applicable | stale (sdd.json changed) | not applicable |
| Invalid entry | not applicable | every lifecycle verb refuses or names it | not applicable |
| `tests/fixtures/` or `.github/workflows/` as an entry, even if also in `ignored_paths` | not applicable | refused (overlaps a meaningful scope) | not applicable |
| A change declaring `--path .` or `--path .chough/` | not applicable | that change refuses at new, answer, approve, check, review and finalize | not applicable |
| `change check --commit` with a tracked `.chough/` edit (step G) | swept into the lifecycle commit | unchanged (out of scope) | unchanged |
| `ignored_paths`, coverage, `exclude_dirs` | unchanged | unchanged | unchanged |

## Contract and compatibility

- **New public policy key**: `verification_ignored_paths` in `.specsync/sdd.json`. Default empty.
- **New JSON field**: `change status --json` `summary.verification_ignored_paths`, omitted when
  empty. No field is added to `verification.json`, `review.json` or any other persisted record.
- **Changed status JSON value, only for an invalid list**: `summary.next_action` names the entry
  to fix. With a valid or absent list, next actions are unchanged.
- **New text lines** in `change status` and `change check`, printed only when the list is
  non-empty, and a new `change ship-status` blocker, printed only when it is invalid.
- **New errors**: an `invalid SDD policy: verification_ignored_paths ...` refusal, and a
  per-change overlap refusal.
- **Contract text**: `change` item 6 is amended to name the exception and item 24 is added;
  `cmd_change` item 12 is added (`deltas/`).
- **With the list absent or empty**, no byte of the digest, persisted evidence or output changes.
- **Older binaries, including 6.1.1, ignore the key** (probe in `context.md`). With a non-empty
  list they compute the full digest, so evidence recorded by a new binary reads as stale to an old
  one. A project whose CI pins an older SpecSync sees review and finalize refused there. That is
  fail-closed, not unsafe, and since no record gains a field, an old binary cannot corrupt new
  evidence by re-serializing it. The docs tell users to raise the CI pin before adding an entry.
- **Archived evidence** is not re-verified against the working-tree digest, so archives are
  unaffected. The implementation confirms this with a test, rather than by assumption.

## Open design questions for the human

1. Projects whose `source_dirs` is `.` or whose `meaningful_paths` contains `.` overlap every
   possible entry, so they cannot use the list. This draft refuses (fail-closed). Relaxing it
   would need a different definition of what the entry must not cover, for example only spec
   `files:`, and is not proposed here.
2. An uncommitted edit under a listed prefix after the review leaves the review current, because
   the review binds the same digest. HI SHIP-2.b says "editing the code after a review makes that
   review stale". Entries cannot overlap source, spec or meaningful paths, so this draft reads a
   listed tool's records as not "the code", but whether that reading is right is the human's call.
   If it is not, the review would have to bind a second, unfiltered digest, which brings back the
   issue for records written between review and finalize.
3. Step G, `change check --commit` sweeping tracked edits under a listed prefix into its lifecycle
   commits, is left out of scope, as the issue suggested. It is better proposed as a follow-up
   under REQ-change-102's staging ownership.
4. The bounds (8 entries, 128 bytes) are proposals.

## From the change's testing.md

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

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
