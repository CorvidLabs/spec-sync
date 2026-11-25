# Lesson bundle: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Check reports a broken lesson file on a cached run, under the module's own name, and when it is not a regular file
- **Kind**: BugFix
- **Specs**: cmd_check, hash_cache, validator
- **Paths**: src/validator/consumed_by_re.rs, src/hash_cache.rs, tests/integration/check.rs, CHANGELOG.md, specs/cmd_check/, specs/hash_cache/, specs/validator/validator.3md
- **Acceptance**: After a clean check, a plain check reports a module's lesson file that was added broken or broken since: every .3md entry beside a spec is an input of its cached verdict and of the changed-spec classification (REQ-hash-cache-011). check reads the lesson file the lifecycle reads, <module>.3md named for the spec's module whatever the spec file is called. A <module>.3md that is not a regular file, such as a directory, is reported as one that cannot be read. change check --strict still fails on the warning, as on any other; REQ-cmd-check-023 and the changelog say that instead of saying lifecycle commands never fail. Each of the three fixes has a test that fails without it, and the full native Verify lane passes.

## Evidence

- Verification commit: `90419f6ae1afd34cf7340d1cd166a76ad4c3001a`
- Base commit: `24d553f6912b7594e8664e629eba5092afd6cbe2`
- Verified by: `specsync check --spec cmd_check --spec hash_cache --spec validator`

## From the change's context.md

# Context

- #1163 added the warning for a lesson file `check` cannot read or parse (REQ-cmd-check-023). Codex reviewed its head `24d553f6` and left four P2 findings on that code. Each was run on the CLI with the binary built from that head before anything was changed.
- Confirmed: after a clean cached `check`, breaking the lesson file is not reported by a plain `check`, because the hash cache does not know the file; `--force` and `--strict` report it.
- Confirmed: for a spec file not named for its module (`main.spec.md`, module `product`), `check` looks at `main.3md` while the lifecycle reads `product.3md`.
- Partly confirmed: a directory named `<module>.3md` is skipped silently. A FIFO or a dangling link there already fails closed in coverage discovery.
- Confirmed, and kept: `change check` passes and `change check --strict` fails on the warning. Strict means warnings fail; the requirement and the changelog said lifecycle commands never fail, which is true of reading lessons and not of a strict check.
- Leif chose to fix the three in one successor change and keep the strict behaviour (release-lead chat, 2026-11-24); he is shown the two deltas before the definition is approved.

## From the change's design.md

# Design

- Hash cache: every `.3md` entry in the spec's directory, listed through `read_project_dir`, joins the spec's companion inputs. That one list feeds the snapshot input digest, the changed-spec classification and the per-file hashes, so no second mechanism is added. The digest also frames each entry's kind, as it does for history fragments, so a file swapped for a directory re-validates.
- All `.3md` entries rather than one name: the cache has the spec's path, not its parsed module, and a superset only costs an extra re-validation.
- Validator: the lesson file is `<module>.3md` from the spec's frontmatter module, falling back to the spec file's stem when the frontmatter has none, which is how the lifecycle names it. The existence test no longer follows the entry: anything of that name goes to the bounded project read, whose refusal becomes the warning.
- Unchanged: the warning's words, the bounded read, coverage discovery's refusal of links, and `change check --strict`.

## From the change's testing.md

# Testing

- Integration, in `tests/integration/check.rs`, each failing before its fix: `a_cached_check_reports_a_3md_broken_since_the_last_run`, `the_3md_checked_is_named_for_the_module_not_the_spec_file`, `a_directory_named_as_the_3md_is_reported`.
- Unit, in `src/hash_cache.rs`: `a_snapshot_is_not_replayed_over_a_changed_lesson_file`.
- The four findings re-run on the CLI with the binary the native lane builds, against the binary of #1163's head.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-cmd-check-023 | `tests/integration/check.rs`: `an_unparseable_3md_is_a_check_warning_and_fails_strict`, `a_3md_over_the_read_bound_is_reported_and_not_read`, `the_3md_checked_is_named_for_the_module_not_the_spec_file`, `a_directory_named_as_the_3md_is_reported`, `a_cached_check_reports_a_3md_broken_since_the_last_run` |
| REQ-hash-cache-011 | `src/hash_cache.rs`: `a_snapshot_is_not_replayed_over_a_changed_lesson_file`; `tests/integration/check.rs`: `a_cached_check_reports_a_3md_broken_since_the_last_run` |

## Where these lessons go

- `specs/cmd_check/cmd_check.3md`
- `specs/hash_cache/hash_cache.3md`
- `specs/validator/validator.3md`
