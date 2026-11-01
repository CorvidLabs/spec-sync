---
change: let-an-opt-in-bounded-verification-ignored-paths-policy-list-keep-another-tool-s-record-directory-out-of-the
artifact: design
---

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
