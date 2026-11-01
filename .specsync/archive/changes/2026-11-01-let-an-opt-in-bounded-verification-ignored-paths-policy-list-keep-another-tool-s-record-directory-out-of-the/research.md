---
change: let-an-opt-in-bounded-verification-ignored-paths-policy-list-keep-another-tool-s-record-directory-out-of-the
artifact: research
---

# Research

## What each existing knob governs today

| Knob | Read by | Effect on the project-input digest |
|------|---------|------------------------------------|
| SDD `ignored_paths` | `path_is_meaningful_with_specs`, `uncovered_paths_error` (`src/change/validate_loaded_change.rs:1113`, `:829`) | none |
| SDD `meaningful_paths` | the same coverage predicate | none |
| `config.toml` `exclude_dirs`, `exclude_patterns` | spec validation, generation and MCP confinement (`src/config.rs`, `src/validator/`, `src/generator.rs`, `src/mcp/`) | none |
| `.gitignore`, `.git/info/exclude` | `git ls-files --exclude-standard` in `git_project_paths` | hides untracked files only |
| Built-in volatile list | `project_input_is_volatile`, `git_path_bytes_are_volatile` | excludes `.git/`, `target/`, `node_modules/`, site build outputs, `.specsync/changes/`, `.specsync/archive/`, the hash cache, the lock and the journal |

## Alternatives considered

1. **Reuse `ignored_paths`.** Rejected.
   - It would give one key two meanings, "not meaningful for coverage" and "not a verification
     input". #676 is the existing evidence that users already misread the first meaning.
   - `ignored_paths` defaults to `[".specsync/", "specs/"]`. Reusing it would silently exclude the
     specs tree and the policy files from the digest in every project. That is the opposite of
     fail-closed.
2. **Reuse `config.toml` `exclude_dirs` / `exclude_patterns`.** Rejected. Those keys accept
   globs, they govern spec validation (which files are scanned for exports), and they are not
   lifecycle policy. Tying the digest to them would let a validation tweak silently widen what
   verification ignores.
3. **Honour Git ignore rules for tracked files.** Rejected. Git itself does not apply ignore rules
   to tracked files, and the user would have no auditable list in SpecSync policy.
4. **Add the directory to the built-in volatile list.** Rejected. That would name one downstream
   tool in product code, which is the same dogfooding problem the BUG-0003 package recorded for
   `change_requires_strict_validation`.
5. **Glob patterns.** Rejected. A pattern such as `**/*.json` or `*/` can exclude specs, sources
   or policy files, and proving that a glob does not do that is much harder than checking prefix
   overlap. Directory prefixes cover the use case in the issue.
6. **A per-change list, declared at `change new`.** Rejected for now. The records belong to a
   tool that runs project-wide, not to one change, and a per-change list would become part of the
   approved scope and need re-approval each time. It remains possible as a follow-up.
7. **Also relax the scoped-review descendant walk for listed prefixes.** Left out of scope. The
   issue asks only for the digest. The walk is a separate review-currency gate tied to HI
   SHIP-2.b, and a sound version would have to read the list from the policy blob at the
   review's implementation commit and convert entries to repository-relative paths with
   `git_repo_relative_path`. It can be proposed as a follow-up.
8. **Bind the digest to `HEAD`'s tree instead of the working tree.** Rejected. That is a much
   larger change to every currency rule, and it would still stale on a committed record (step F).
9. **Opt-in, bounded policy list (`verification_ignored_paths`).** Chosen. It is separate from
   `ignored_paths`, empty by default, held in a protected policy file, refused by every lifecycle
   verb that loads the policy, bound into evidence through the project-input digest, and disclosed
   in status.

## Bounds, and why each one

- **Directory prefixes ending in `/` only.** A prefix is the only shape whose overlap with
  `.specsync/`, the specs tree, `source_dirs` and spec `files:` can be decided exactly. A bare name
  such as `.chough` (no slash) would also match a file or symlink named `.chough`, so the bare
  path is never excluded.
- **Portable relative components.** The same rules as `strict_portable_relative_path`
  (`src/change/collect_requirement_ids.rs:3795`): no leading `/`, no drive letter, no backslash,
  no control character, no empty, `.` or `..` component.
- **No pattern characters** (`*`, `?`, `[`, `]`, `{`, `}`, `!`). They are refused outright rather
  than treated as literal names, so an entry never looks like a glob that does not work.
- **No symlink at or above the entry.** Git lists a symlink as a single path, and a symlinked
  directory could point into `src/`. An entry whose path, or any parent of it, is a symlink on disk
  is refused. An entry that does not exist on disk yet is allowed, because the tool may not have
  written its first record yet.
- **Count at most 8, each at most 128 bytes, no duplicate, no entry nested in another.** These keep
  the list small enough to audit by eye and make the stored list order-independent.
- **No overlap with protected inputs.** An entry may not equal, contain or lie inside:
  - `.specsync/`,
  - the configured specs tree,
  - a `source_dirs` entry,
  - the configured `schema_dir`,
  - a protected SDD policy file (`is_protected_sdd_path`),
  - `principles_file`,
  - a `custom_artifacts` template.

  It may also not contain any file listed in a spec's `files:`, any path-form spec `depends_on`
  entry, or any file named in a spec's `Consumed By` table. A `source_dirs` entry of `.`
  overlaps every entry, so such a project cannot use the list (see the open question in
  `design.md`).
- **No overlap with any `meaningful_paths` scope, whatever `ignored_paths` says.** An entry is
  refused when it equals, contains or lies inside a `meaningful_paths` scope. `ignored_paths` is
  never consulted. An earlier revision accepted an entry inside a meaningful scope when an
  `ignored_paths` scope also covered it. That made `ignored_paths` a switch for digest exclusion,
  the second meaning #676 warns about, and it would have let `tests/fixtures/` or
  `.github/workflows/` (both under default meaningful scopes, and both read by tests or CI) leave
  the digest after one extra `ignored_paths` line. Refusing outright costs projects with
  `meaningful_paths: ["."]` the feature (open question 1 in `design.md`). Under the default
  policy, `.chough/` overlaps no meaningful scope and passes.
- **Project-relative entries.** Entries are compared with the project-relative paths that
  discovery already produces (`git ls-files` runs from the project root). No repository-relative
  path is compared against an entry in this change.
- **Case-insensitive overlap.** On case-insensitive file systems `SRC/` and `src/` are one
  directory. Overlap is refused when the entries are equal ignoring ASCII case. Exclusion itself
  matches only the exact case Git reports, so a case trick can only *include* more paths.
- **No overlap with a change's declared `affected_paths`.** A change that declares `--path .` or
  `--path .chough/` claims those files as its scope, and its acceptance evidence hashes them. The
  workspace digest must not ignore what the change itself claims.

## The limit SpecSync cannot close

`change check` compares specs to code in-process and runs no project command (REQ-change-023,
REQ-change-091). The validation reads specs, the files those specs list, and SDD and config
policy. All of those are refused as entries, so the in-process verdict cannot depend on an
excluded file.

The project's own tests and CI can still read excluded files, and SpecSync cannot see that. If they
do, editing those files leaves verification current without anyone re-running those tests. The
docs say so plainly, and `change status` and `change check` repeat it as a standing note while the
list is non-empty.

The list also narrows one reading of HI SHIP-2.b: an uncommitted edit under a listed prefix after
the review leaves the review current. That is open question 2 in `design.md`.

## Evidence-shape compatibility

- `SddPolicy` is `#[serde(default)]` and tolerant of unknown keys (`src/change/sdd_version.rs:671`).
  The new field defaults to empty. Policies written before it parse unchanged, and today's binaries
  ignore the key (probe in `context.md`).
- `VerificationRecord` gains no field. It is read tolerantly, but its bytes are compared byte for
  byte when accepted evidence is anchored (`consider_accepted_evidence_anchor.rs:2132`-`:2153`)
  and feed `accepted_evidence_key`. A field omitted when empty would keep today's bytes, but a
  non-empty one would be dropped by an older binary that re-serializes the record, and that would
  change the anchored bytes. The verdict already binds the list through `workspace_digest`.
- `ScopedReviewRecord` and the finalization record are digest preimages (finalization
  `review_digest`), so they gain no field. Both already bind the list through `workspace_digest`,
  which covers the bytes of `.specsync/sdd.json` and the list frame.
- `ChangeSummary` gains `verification_ignored_paths`, omitted when empty. It is output only.
