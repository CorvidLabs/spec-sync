---
id: let-an-opt-in-bounded-verification-ignored-paths-policy-list-keep-another-tool-s-record-directory-out-of-the
state: implementing
type: feature
base_commit: a80748ff44ff42cd7578ac0f7cb8ba7e080ab183
---

# Let an opt-in bounded verification_ignored_paths policy list keep another tool's record directory out of the verification project-input digest

## Intent

Let an opt-in bounded verification_ignored_paths policy list keep another tool's record directory out of the verification project-input digest

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- With `.specsync/sdd.json` carrying `"verification_ignored_paths": [".chough/"]` committed and the change re-verified, adding, editing or deleting a file under `.chough/`, tracked or untracked, and committing such files before the review, leaves the project-input digest unchanged: `change status <id> --json` keeps `summary.next_action` at `specsync change review`, `change review` records a passing review, and `change finalize` accepts it. On the built binary this is the issue #825 repro with the list in place of the workarounds it tried: step B (an untracked `.chough/evidence/obs.json` after verification), step D (the list committed, the change re-verified, then the record restored) and step F (the record committed before the review) each leave status naming `change review`.
- The same additions, edits, deletions and commits outside every listed prefix (for example `notes/obs.json` or `src/lib.rs`) still stale verification and review exactly as today, and a path equal to a listed prefix without its trailing slash (a file or symlink named `.chough`) is not excluded. The scoped-review descendant walk is unchanged: any commit after the review, including one that touches only paths under a listed prefix, stales the review as today. An uncommitted edit under a listed prefix after the review leaves the review current, because the review binds the same project-input digest.
- An entry is refused with a message naming the entry and the rule it breaks unless it is a portable relative directory prefix ending in `/`: no glob or pattern characters, no absolute path, no empty, `.` or `..` component, no backslash or control character, no symlink at or above the entry on disk, at most 128 bytes, no duplicate or nested entries, and at most 8 entries.
- An entry is refused when it equals, contains or lies inside `.specsync/`, the configured specs tree, a configured `source_dirs` entry, the configured `schema_dir`, any `meaningful_paths` scope, an SDD policy file, the configured `principles_file` or a `custom_artifacts` template, or when it contains a file listed in any spec's `files:`, a path-form `depends_on` entry of any spec (one naming a path, not a bare module name or a cross-project reference) or a file named in any spec's `Consumed By` table. `ignored_paths` does not relax this: an entry overlapping a `meaningful_paths` scope, such as `tests/fixtures/` or `.github/workflows/` under the default policy, is refused even when an `ignored_paths` scope also covers it. Overlap is compared ignoring ASCII case. Entries are project-relative, so a project rooted in a subdirectory of its Git repository excludes only paths under `<project>/<entry>`. The list never changes which paths are meaningful.
- An entry that overlaps a declared `affected_paths` scope of a change is refused for that change at `change new`, at `change answer <id> affected_paths`, and at `change approve`, `change check`, `change review` and `change finalize`, naming the entry and the change's path.
- An invalid list fails closed in every lifecycle verb that loads SDD policy, naming the entry and the rule it breaks: `change new`, `change answer`, `change approve`, `change check`, `change review`, `change finalize`, `change ship` and `change audit` refuse; `change status` makes fixing that entry its one next action; and `change ship-status` reports it as a blocker naming the entry rather than as stale verification. No evidence is recorded against an invalid list. `specsync check` does not read the list.
- Adding, removing or editing an entry after verification stales verification and review, because `.specsync/sdd.json` is a project input. No persisted record gains a field: `verification.json`, `review.json` and finalization evidence bind the list only through the bytes of `.specsync/sdd.json` and the list frame in the project-input digest. When the list is non-empty, `change status --json` reports it as `summary.verification_ignored_paths`, and `change check` and `change status` text print it with a standing note that SpecSync cannot tell whether project tests or CI read files under those prefixes.
- With the list absent or empty, the project-input digest, `verification.json`, `review.json`, status JSON and every other persisted byte are identical to origin/main a80748ff, and `ignored_paths`, `meaningful_paths`, `config.toml` `exclude_dirs` and `exclude_patterns`, coverage, acceptance-input digests, the scoped-review descendant walk and `change check --commit` staging are unchanged.
- Regression tests are labelled DISCRIMINATOR (fail on origin/main a80748ff) or CONTROL (pass on both), and include an integration test through the built binary that, with the list set, replays issue #825 steps B, D and F as named in the first criterion, records a passing review and finalizes, and a second that shows a commit after the review touching only the listed prefix still stales the review.

## No-spec Rationale

Not applicable
