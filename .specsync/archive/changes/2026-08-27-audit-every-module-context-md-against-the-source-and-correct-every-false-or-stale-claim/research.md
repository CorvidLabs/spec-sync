---
change: audit-every-module-context-md-against-the-source-and-correct-every-false-or-stale-claim
artifact: research
---

# Research

Every number below is one this change measured, with the command that produced it, so the next
reader can re-derive it rather than inherit it. That is exactly what failed in #714.

## Counts measured

| claim | command | result |
|---|---|---|
| tracked `.md` files, all LF | `git ls-files '*.md' \| wc -l`, then a per-file `grep -q $'\r'` and BOM probe | **2263**, none with CR, none with BOM (context said 2103) |
| archived approval ledgers | `find .specsync/archive/changes -name approvals.json \| wc -l` | **202** |
| ledgers with no `approved_delta_digests` on any definition gate | JSON scan of all 202 | **188** (context said 183) |
| ledgers with the shape #719 rejects (some definition gate records a digest, the LAST does not) | same scan | **0** — the refusal is still safe on history |
| unit tests | `cargo test --bins -- --list` | **2407** (contexts said 1,953 / 1,954) |
| integration tests | `cargo test --test integration -- --list` | **407** (contexts said 312 / 313) |
| source files directly under `src/exports/` | `ls src/exports/*.rs \| wc -l` | **34**, plus 12 under `ast/` (context said "~14") |
| jobs in `ci.yml`, all ubuntu | `grep -c 'runs-on:' .github/workflows/ci.yml` | **16**, all `ubuntu-latest` — the "sixteen jobs" claim is TRUE |
| `.replace("\r\n", "\n")` occurrences in `src/` | `grep -rno '\.replace("\\r\\n", "\\n")' src --include='*.rs' \| wc -l` | **29** — the count the wrong lesson was built from is reproducible |
| `parse_frontmatter(` call sites outside `parser.rs` | `grep -rn 'parse_frontmatter(' src --include='*.rs' \| grep -v parser.rs \| grep -v change_tests.rs` | **39** |
| integration tests invoking `report` | `#[test]`-block scan for `"report"` as an argv element | **17** (context said there were none) |
| integration tests invoking `diff` | `grep -rn 'fn diff_' tests/integration/*.rs` | **9** (context said five) |
| integration tests invoking `coverage` | `#[test]`-block scan for `"coverage"` as an argv element | **34** (context said 8) |
| inline `#[test]` fns in `src/commands/init.rs` | `grep -c '    #\[test\]'` | **12** (context said three) |

## The `21 of 39` split: measured, and deliberately left alone

`specs/parser/context.md`, `specs/change/context.md` and `src/parser.rs:300` all state that 21 of
the 39 `parse_frontmatter` call sites outside `parser.rs` normalize and 18 do not. Re-measured:

- The **denominator is exact**: 39 today, and `git grep -n 'parse_frontmatter(' e302ae31 -- src`
  gives 39 with an identical per-file distribution. The call-site set has not moved since #696
  measured it, so nothing drifted.
- The **split is definition-sensitive**. Counting only site-local normalization (an inline
  `.replace` or a binding assigned in the same function from one) gives 22/17; following rebinding
  chains and helper functions that normalize (`read_spec_status`, `read_file_at_ref`) gives 24/15.
  No command recorded anywhere reproduces exactly 21/18.

Every reading agrees on the claim the sentence actually makes — roughly half normalize and half do
not, so there is no convention — and no reading shows 21/18 to be wrong by more than three. It was
left unchanged. Replacing a figure that is within noise of three plausible methods with one of
mine, and calling that a correction, would be the #714 mistake with the sign flipped. The residual
worth naming: **the corrected count is itself a number without its command**, which is the same
shape of defect the correction was fixing.

## Behavioural claims settled by reading the source

- `floor_sequence_ledger_to_committed` (`src/change.rs:1869`) calls `write_json` on
  `.specsync/change-sequence.json`, and `git_commit_all` (`src/commands/change.rs:2865`) calls it
  before every lifecycle `git add -A`. "Nothing writes it any more" is false; nothing *allocates*
  in it, which is a different claim.
- `check_project_quiet` was deleted by `4ddf810e` (#543). `check_project` and `audit_project` now
  share the private `check_project_with_command_output`. `src/commands/comment.rs` has no
  `crate::change` reference at all and states at line 58 that lifecycle reporting was removed.
- `.specsync/change-sequence.json` records **five** acknowledged collisions (16, 48, 49, 99, 100),
  not the single `CHG-0016` the context named.
- `auto_regen_stale_specs` was removed by `884ad33b` (#335) — the same commit that last touched
  `specs/cmd_check/context.md`, which still lists it.
- `.github/workflows/finalize-change.yml`, `lifecycle-policy-guard.yml`,
  `post-merge-archive.yml` and three Python verifiers were deleted by `802ca13b` (#499), ~7,257
  lines. `grep -rn pull_request_target .github/workflows/` returns nothing. Four `specs/github/context.md`
  Key Decisions bullets described them in the present tense.
- `src/exports.rs` has never existed (`git log --all -- src/exports.rs` is empty); it is
  `src/exports/mod.rs`. Four context files cited the path, and each also named a function the
  module in question does not call (`has_extension` vs `has_configured_extension`,
  `get_exported_symbols` vs `get_exported_symbols_full` / `scan_exported_symbols_full`).
- `src/registry.rs:216` parses with `toml::from_str`; the context claimed line-by-line parsing.
  `Cargo.toml:38` has depended on `toml` since #483, which also falsifies the `specs/agents`
  claim that no `toml` dependency exists.
- `src/output.rs:606` calls `std::fs::read_to_string` inside `print_diff_markdown`; the context
  claimed the module does no file I/O.
- `src/commands/deps.rs` runs `validate_deps` in diagram mode and exits 1 on errors; the context
  said diagram mode returns early without validation.
- `output::percent_json` returns `Value::Null` for a zero denominator (#582); `specs/cmd_coverage`
  said a zero denominator is treated as 100%.
- `remove_section` has never existed in `src/hooks.rs`; the function is `remove_section_from_file`.
- `is_suppressed` / `suppression_source` are called from `src/commands/mod.rs`,
  `src/commands/issues.rs` and `src/change.rs`. `src/validator.rs` does not reference `ignore` at
  all, though `specs/ignore/context.md` named it as a call site.

## Examined and deliberately NOT changed

- `21 of 39` (above).
- `all 197 ledgers were scanned before the refusal was written` — past tense about an action
  performed then, and true of it; 202 exist now.
- `the descendant walk ... passes 0 of 107 archived reviews (#694)` — attributed to the issue that
  measured it; 131 archived reviews exist now.
- `two archived deltas contain duplicate MODIFIED keys` — simulating the pre-#564 flush ordering
  over all 452 archived deltas reproduces exactly two files that predate the fix. Under the
  current parser the count is 0, which is the point the sentence is making.
- `counted from 29 occurrences` — reproducible today at 29. (#714's own text says 28; the issue is
  the one that is off by one, not the lesson.)
- Partial field lists (`CompactResult`, `DepsReport`) that name only real fields and claim no
  exhaustivity. `DepsReport` was extended anyway because the two omitted fields drive
  `disclosures()`, which a reader of the diagram path needs.
- "45 focused validator tests" / "52 focused manifest tests" — change-evidence counts, not module
  totals. Current module totals are 64 and 59.
