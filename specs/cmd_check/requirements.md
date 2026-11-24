---
spec: cmd_check.spec.md
---

## User Stories

- As a developer, I want `specsync check` to validate every spec against its source so that drift is caught before it ships
- As a developer on a large repo, I want unchanged specs skipped via a hash cache so that repeated checks stay fast, and `--force`/`--no-cache` to re-validate everything when I need it
- As a CI operator, I want exit codes driven by enforcement mode, `--strict`, and `--require-coverage` so that pipeline pass/fail is predictable
- As a developer, I want `--fix` to make safe deterministic markdown repairs while leaving contract judgment to me or my coding agent
- As a cautious developer, I want `--dry-run` to preview `--fix` and `--backup` to snapshot specs before they are rewritten so that I never lose work
- As a maintainer, I want `--stale [N]` to flag specs that are N+ commits behind their source files so that I can spot quietly-rotting docs
- As a tool integrator, I want `--format json/markdown/github` so that results render in dashboards, PR bodies, or Actions logs

## Acceptance Criteria

- Validates discovered specs, applying `--exclude-status`/`--only-status` and positional `[SPEC...]` filters
- A hash cache (`.specsync/hashes.json`) skips re-validation of unchanged specs unless `--force`/`--no-cache`, `--strict`, or explicit spec filters are given; the skipped count is reported in text mode, and any stored findings are replayed so a warm run cannot go silent
- Requirements drift remains visible as validation guidance for humans and coding agents
- `--fix` renames near-miss headers and appends undocumented exports with language-aware skeleton rows; it performs no inference or command execution
- `--backup` copies specs to `.specsync/backup-fix/` before any `--fix` write, aborting on any copy/dir failure to avoid data loss
- `--dry-run` previews `--fix` without writing; `--dry-run` without `--fix` prints a warning that it has no effect
- `--stale [N]` (default N=5) uses `spec_baseline` + `git_commits_since` to count how many commits each source file has advanced past the spec's last commit, flagging specs ≥ N behind. When the tree has no committed history — no repository, or an unborn `HEAD` — it emits one `history_unavailable` warning saying it could not check, rather than silently producing no output (#572); a warning is the same weight a real drift finding carries here, so `--strict` fails and a plain run still exits 0
- `--create-issues` creates one GitHub issue per spec with errors (only when `total_errors > 0`)
- The hash cache is updated and saved only when `total_errors == 0`
- JSON output is a single object with `passed`, `errors`, `warnings`, `stale`, and `specs_checked`
- Exit code comes from `compute_exit_code`/`exit_with_status` (Warn/EnforceNew/Strict + require-coverage)
- When `.specsync/sdd.json` enables SDD, unified check first validates change coverage, approvals, semantic conflicts, and code against the effective canonical-plus-approved-delta contract.

## Constraints

- Must not panic on expected error conditions — print and exit
- `--fix` only modifies spec markdown files, never source code
- `--fix` is deterministic and local regardless of inference-related environment variables
- `--stale` is a no-op outside a git repo (no crash); specs without a `files:` list are skipped
- Git staleness uses `git_commits_since` (single rev-list per file) — the earlier per-pair `git_commits_between` N+1 walk was removed

## Out of Scope

- Modifying or generating source code
- Defining the CLI grammar (lives in `src/cli.rs`)
- The scoring algorithm itself (lives in the scoring module; check only renders `--explain` breakdowns)
- Non-git staleness heuristics (filesystem mtimes, etc.)

### REQ-cmd-check-001

The `cmd_check` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-cmd-check-002

The check and fix pipeline SHALL remain deterministic and local.

Acceptance Criteria
- `--fix` performs deterministic markdown repairs and never invokes an embedded model or shell AI command.
- Requirements drift remains visible as validation guidance for a coding agent to resolve.
- Existing cache, enforcement, lifecycle, output-format, backup, and dry-run behavior remains intact.

### REQ-cmd-check-003

The primary check command SHALL consume one fallible schema snapshot and report warning
suppression truthfully in every supported output.

Acceptance Criteria

- Schema replay, table identity, pattern additions, and column validation derive from one immutable
  snapshot per validation invocation.
- Missing, unreadable, malformed, or vacuous configured schema input is an explicit finding and
  cannot become an empty successful comparison.
- Text, JSON, Markdown, and GitHub output distinguish emitted warnings from deterministic
  `suppressed_warnings` details.
- Strict exit behavior counts unsuppressed findings only and preserves existing cache and coverage
  semantics.

### REQ-cmd-check-004

The primary check command SHALL provide bidirectional spec-to-code drift validation. With explicit `--strict` on a Git project that has SDD enabled in its working tree or at its trusted comparison base, it SHALL also run the shared branch comparison in REQ-change-archived-requirement-loss-001 and account for a loss using approved removals. It SHALL report the historical findings through its supported text and structured validation results. Lifecycle policy compliance, general workspace health and unrelated archive integrity SHALL remain with the lifecycle commands.

Acceptance Criteria

- Ordinary `specsync check` reports drift without emitting lifecycle workspace or archive findings.
- After the last change has been archived, `check --strict` exits unsuccessfully if the branch dropped a canonical requirement without an approved removal. It names the lost ID, the original canonical file, and how to display the original block.
- A separate managed fixture with no requirement loss passes. A managed project can use only strict check in CI and still catch an unexplained loss.
- A branch that sets `enabled` to false or deletes `.specsync/sdd.json` while its comparison base has SDD enabled is still compared. `check --strict` and `change audit --strict` both refuse an unexplained loss there.
- A Git project that never enabled SDD, or whose comparison base has it disabled, retains the ordinary spec validation result, including when a requirement was edited without a removal delta. The same applies outside Git.

### REQ-cmd-check-005

`specsync check` SHALL report requirements drift and companion updates only for
classifications observed against a known baseline.

Acceptance Criteria
- A project with no hash cache reports no requirements-drift warnings in any output format.
- The warning count, the machine-readable staleness entries, and the review hint all follow the same condition.
- Spec selection is unaffected: the same specs are re-validated whether or not a baseline exists.

### REQ-cmd-check-006

Machine-readable check output SHALL carry the skipped symlinked entries.

Acceptance Criteria
- The JSON payload includes the full list of skipped entries, not a truncated summary.
- The field is present whenever the payload reports a result.

### REQ-cmd-check-007

Validation of a project with no specs SHALL report the coverage it measured, and SHALL NOT
report a tree containing unmeasured source as clean under strict validation.

Acceptance Criteria
- The coverage figures are printed whenever there are no specs to validate, not only when a gate has already failed.
- Strict validation exits non-zero when the project contains source files and no specs.
- A project with no source files continues to exit zero under strict validation.
- The machine-readable payload carries the source-file count and coverage percent, so a project with unmeasured source is distinguishable from an empty one.

### REQ-cmd-check-008

An automatic fix that could not be applied SHALL be reported, and SHALL NOT be reported as
success.

Acceptance Criteria
- A spec that cannot be written is reported with its path and the underlying error, and the command exits non-zero.
- A spec that cannot be read is reported the same way rather than skipped silently.
- Failures are reported in every output format, so a machine consumer that requested a mutation is not left reading only a success payload.
- A writable spec is still repaired and the command still exits zero.
- A dry run attempts no write and therefore reports no write failure, exiting zero even when the target is not writable.

### REQ-cmd-check-009

`check` SHALL report an unmeasured tree as unmeasured in every format.

Acceptance Criteria
- Text, JSON, CSV, Markdown, GitHub and table all decline to print a percentage.
- JSON uses `null`.

### REQ-cmd-check-010

`check --stale` SHALL refuse when staleness cannot be measured.

Acceptance Criteria
- An explicitly requested measurement that cannot be taken exits non-zero rather than emitting zero warnings.
- JSON reports `null`, never an empty list.
- Plain `check` without `--stale` is unaffected.

### REQ-cmd-check-011

`check` SHALL name every finding in table and csv.

Acceptance Criteria
- csv emits one row per finding with stable columns; a clean tree emits a header and no rows, distinguishable from a run that never happened.
- table emits an aligned list.
- Staleness findings appear in every non-text format, not only the tabular pair — they drive the exit code, so a format that omits them exits non-zero while naming nothing.

### REQ-cmd-check-012

`check` SHALL NOT drop findings because the hash cache is warm.

Acceptance Criteria
- A second run over an unchanged tree reports the same warning identity the first run reported.
- JSON `specs_checked` is not 0 when a spec exists and was either validated or replayed.
- A spec is skipped only when a replayable validation snapshot still binds to the current inputs.
- `--force` / `--no-cache` still re-validate and still report the same identity.
- An in-sync tree remains all-clear on both the cold and the warm run.

### REQ-cmd-check-013

`check` SHALL produce identical findings on repeated runs over an unchanged tree.

Acceptance Criteria
- Two consecutive runs report the same findings in text and in JSON.
- A skipped spec is counted in `specs_checked` and its warnings are named.
- A genuinely clean spec still reports clean, so replay does not manufacture findings.
- `--force` and `--no-cache` are unaffected.

### REQ-cmd-check-014

`check` SHALL disclose cited files whose drift it could not measure, including on specs where other files were measured.

Acceptance Criteria
- A cited file that is absent or is a directory is named as unmeasurable rather than skipped without record.
- The disclosure appears even when the spec's other files produced a drift number, so that number is not read as covering the whole spec.

### REQ-cmd-check-015

`check --fix` SHALL insert each new export row immediately after the last row of the target Public API table that REQ-cmd-check-017 selects in the matching section or subsection, never after trailing prose, and SHALL append rows at the end of the section only when that section contains no table.

Acceptance Criteria
- A table under a `### Heading`, a `**Bold**` label, or no label followed by prose receives the row inside the table and the prose is preserved after it.
- A `## Public API` section with no table still receives rows at its end, as before.
- `check --strict` passes on the rewritten spec without further edits.

### REQ-cmd-check-016

`check --fix` SHALL ignore fenced Markdown when locating `###` export subsections and near-miss headings. A fenced `### Exported Functions` is quoted text, not an insertion target. Near-miss and bare-kind header rewrites SHALL scan a fence-blanked copy and SHALL write the original section, so a fenced example body is never replaced with spaces.

Acceptance Criteria
- A Public API section whose first `### Exported Functions` is inside a fence, followed by a real heading and table, receives new rows in the real table.
- The fenced sample is preserved verbatim, including when `--fix` also renames a near-miss or bare `###` heading in the same section.
- A fenced `### Exported Functons` is not renamed.

### REQ-cmd-check-017

`check --fix` SHALL insert every new export row for one Public API block into exactly one target table, chosen as follows, and SHALL take the row's column count from that same table: the last table in the block that holds a documented export row; otherwise the first table whose first-column header is export-like and which is not a parameter or field table; otherwise the last table in the block, as before.

Acceptance Criteria
- A block is one `###` export subsection, or the flat `## Public API` section when it has no `###` subsections. A table is a maximal run of consecutive table-row lines outside fenced or indented code; the fence and indentation rules are unchanged.
- A documented export row is a data row below the separator from which `get_spec_symbols` reads a symbol: the first non-empty backticked name in its first cell.
- An export-like first-column header is `Name`, `Function`, `Export` or `Symbol`, singular or plural, trimmed and compared ignoring case. A parameter or field table is one whose header has a column other than the first named `Type`, or whose first-column header is `Parameter`, `Param`, `Argument` or `Field`.
- Width and insertion point never come from two different tables.
- An export table `| Name | Kind | Description |` holding `a`, followed by prose and `| Parameter | Description |` (with or without a parameters table before it, under a heading, a bold label or no label, and with `Function` in place of `Name`) receives a 3-cell row for `b` directly after the `a` row (#768 case 1).
- `| Parameter | Description |` followed by `| Function | Description |` holding `a` receives a 2-cell row for `b` directly after the `a` row.
- `| Function | Description |` holding `a`, followed by prose and `| Name | Type | Description |`, receives a 2-cell row for `b` directly after the `a` row.
- In each case every other byte of the file is unchanged, `check --strict` passes, and a second `check --fix` changes nothing.

### REQ-cmd-check-018

When the hash cache location is refused (REQ-hash-cache-007), `check` SHALL print one warning on stderr that names the refused path, SHALL validate every spec as it does with no cache, SHALL NOT read, create or write anything through that location, and SHALL keep the stdout output, verdict and exit code it has with no cache.

Acceptance Criteria
- With `.specsync/hashes.json` a symbolic link to a file outside the project, `specsync check`, `check --force`, `check --strict` and `check --format json` each exit as they do without the link, print `hash cache not used` once on stderr, and leave the link and the bytes of its target unchanged.
- With that link dangling, nothing is created at its target.
- With `.specsync` a symbolic link to a directory outside the project, `specsync check` prints the same warning and that directory gains no entry.
- A project whose cache is a regular file or absent prints no such warning.

### REQ-cmd-check-019

`check` SHALL read the spec files it reads again for `--fix` and for stale-history detection only through the bounded project read of `util` (REQ-util-003), with a bound of 8 MiB. A read refused there, because the file leaves the project through a symbolic link, is a dangling link, is not a regular file, or is larger than the bound, SHALL be handled as that read's other read errors are, with a message that names the path and the reason, and SHALL never wait on the file or read past the bound.

Acceptance Criteria
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no direct file read in this module's non-test source outside the allowed list

### REQ-cmd-check-020

`check --fix --backup` SHALL read each spec through the bounded project read and write its backup to `.specsync/backup-fix/<spec path>` through the confined project write (REQ-util-002). A `backup-fix` directory, or a file in it, that leads out of the repository SHALL be refused with the path, and `--fix` SHALL stop before it changes a spec. `--fix` SHALL rewrite specs through the confined project write.

Acceptance Criteria
- A `backup-fix` linked to an outside directory receives nothing, and the spec is unchanged; a backup file linked to an outside file keeps that file's bytes.
- With no link, the backup is written where it was and the fix goes on.

### REQ-cmd-check-021

`check` SHALL use only the hash cache in the project's local state folder (REQ-hash-cache-009) and SHALL never read a `.specsync/hashes.json` in the work tree or a cache under a Git directory, so a cache a repository or an archive of one ships changes nothing and prints nothing. The "Cache:" line SHALL name the cache's real path. When no local state folder can be found, or it is refused (REQ-util-006), `check` SHALL print once `warning: hash cache not used:` with the reason and validate every spec.

Acceptance Criteria
- With a forged `.specsync/hashes.json` in a tree with no Git, `check` validates the failing spec and exits 1.
- A copy of a project that ships its `.git`, cache included, validates every spec.
- With `XDG_CACHE_HOME` inside the project, `check` warns once and validates every spec.
- With the project's state folder writable by others, `check` warns once and validates every spec.

### REQ-cmd-check-022

`check` SHALL list the schema folder for its cache inputs only through `read_project_dir` (REQ-util-010).

Acceptance Criteria
- A schema folder that leads out of the project contributes no input name from outside.

### REQ-cmd-check-023

`check` SHALL report a module's `<module>.3md` lesson file that exists beside its spec but cannot be read or parsed as 3md as a warning on that spec, naming the file, the parser's error, and that the module's `context.md` is used for its lessons instead, so `check --strict` fails on it. The file is the one the lifecycle reads: it is named for the spec's module, whatever the spec file is called, and an entry of that name that is not a regular file SHALL be reported as one that cannot be read. A `.3md` that parses, and a module without one, SHALL add no finding. Reading a module's lessons still falls back to `context.md` and never fails a lifecycle command (REQ-change-106); a strict lifecycle check, `change check --strict`, fails on this warning as on any other.

Acceptance Criteria
- A spec whose `<module>.3md` has two planes at the same `z` gets one warning naming the file, the parser's error and `context.md`, and `check --strict` exits non-zero.
- A spec whose `<module>.3md` parses, or that has none, gets no such warning, and `check --strict` passes for it.
- A spec file not named for its module, such as `main.spec.md` for module `billing`, is checked against `billing.3md`; a `main.3md` beside it adds no finding.
- A directory named `<module>.3md` gets the warning, and `check --strict` exits non-zero.
- After a clean `check`, a `check` with no flags reports a lesson file that was added broken or broken since (REQ-hash-cache-011).

