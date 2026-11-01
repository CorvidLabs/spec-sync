# Lesson bundle: write-generated-markdown-and-hook-files-without-em-or-en-dashes-while-still-recognizing-the-managed-markers-earlier

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Write generated Markdown and hook files without em or en dashes while still recognizing the managed markers earlier versions wrote
- **Kind**: BugFix
- **Specs**: hooks, agents, compact, change, cmd_init, cmd_migrate
- **Paths**: src/hooks.rs, src/agents.rs, src/compact.rs, src/change/historical_definition_digest_matches.rs, src/commands/init.rs, src/commands/migrate.rs, tests/integration.rs, tests/integration/, .claude/commands/specsync/create-spec.md, .cursor/commands/specsync-create-spec.md, .claude/skills/spec-sync/SKILL.md, .codex/skills/spec-sync/SKILL.md, .cursor/skills/spec-sync/SKILL.md, .gemini/skills/spec-sync/SKILL.md, .specsync/agent-artifacts.json, specs/hooks, specs/agents, specs/compact, specs/change, specs/cmd_init, specs/cmd_migrate
- **Acceptance**: After `specsync change finalize`, the archived `lesson-bundle.md` opens with `# Lesson bundle: <id>`, and every U+2013 or U+2014 it contains comes from user-supplied record text that the bundle copies verbatim: the change's title, acceptance criteria, affected specs and paths, recorded verification command strings, and the bodies of its context.md, design.md and testing.md. The bundle's own fixed text and heading add neither character.
- **Acceptance**: `specsync init`, `specsync migrate` (3.x to v4 layout), `specsync hooks install` for every target (CLAUDE.md, .cursorrules, .github/copilot-instructions.md, AGENTS.md, the Git pre-commit hook, .claude/settings.json), `specsync agents install` for every tool, and `specsync compact` summary rows write no U+2013 or U+2014.
- **Acceptance**: The managed instruction block begins with `<!-- specsync:hook:begin (managed by `specsync hooks`; do not edit inside) -->` and still ends with the unchanged `<!-- specsync:hook:end -->`. `specsync hooks uninstall` removes a block whose begin line is the 6.1.1 form exactly as it removes the new form, preserving content before and after it; `hooks install` and `hooks status` keep treating either form as installed.
- **Acceptance**: `specsync hooks uninstall --precommit` still removes a project-keyed managed block whose body 6.1.1 wrote, found by its unchanged markers. Recognition of unmarked legacy hooks (an appended block after `# --- spec-sync pre-commit hook ---`, or a whole-file hook) keeps comparing against the current hook body, as today; no frozen copy of an older body is added.
- **Acceptance**: A compaction summary row is written as `| <first> to <last> | - | Compacted: N entries <!-- specsync:compact:v1 --> |`. A summary row in the 6.1.1 form (a U+2014 range separator surrounded by spaces, and U+2014 placeholder cells) is still recognized as the prior summary: a no-excess re-run leaves the file byte-for-byte unchanged, and a run that compacts more entries folds its count and range start into one new-form row.
- **Acceptance**: `AGENT_ARTIFACT_TEMPLATE_VERSION` advances to 7, so `specsync agents install` refreshes create-spec command files whose bytes still match their recorded digest, and the repository's own installed copies and `.specsync/agent-artifacts.json` are regenerated from the built binary.
- **Acceptance**: A regression test drives every writer in the second criterion (init, migrate, hooks install, agents install and compact) through the built binary in a disposable repository and fails if any file it writes, other than user-supplied input, contains U+2013 or U+2014. It fails on origin/main a80748ff and passes with the fix. The lesson bundle is covered by an in-process unit test of `write_lesson_bundle`, because a binary-driven finalize needs a fully approved and reviewed change.
- **Acceptance**: Unchanged: terminal, JSON and stdout Markdown output (including `--format markdown` reports, `specsync comment` bodies and issue verification text); the `Spec-Sync Integration` and `Spec-Sync Rules` install-detection markers; the end marker and pre-commit project-keyed markers; frozen archives under `.specsync/archive/`; existing installed blocks and compacted rows, which are not rewritten until the user reinstalls or compaction next folds them.

## Evidence

- Verification commit: `77380cd0d2e739fd9bd0003c59a56812f10871ff`
- Base commit: `a80748ff44ff42cd7578ac0f7cb8ba7e080ab183`
- Verified by: `specsync check --spec agents --spec change --spec cmd_init --spec cmd_migrate --spec compact --spec hooks`

## From the change's context.md

# Context

Issue #827, reported downstream as CorvidLabs/chough#13. SpecSync writes U+2014 (em dash) into
files it generates in the user's repository even when none of the user's inputs contain one.
CorvidLabs documentation rules forbid em and en dashes in prose, and at least one CorvidLabs
repository lints tracked and untracked Markdown for them. A project that uses SpecSync therefore
fails its own docs gate because of SpecSync output. Some of that output cannot be fixed by hand:
the managed hook block says "do not edit inside", and the lesson bundle lives in a frozen archive
that downstream tools such as Chough must not edit.

## Reproduction (installed 6.1.1 and origin/main a80748ff)

The issue's `base.sh` and `repro3.sh` were extracted verbatim from the issue body and run in a
disposable directory twice: with the installed `specsync 6.1.1`, and with a release build of
origin/main a80748ff (`cargo build --release` in this worktree, put first on `PATH`). The fixture
was the local Chough checkout at 2756dc2 (`FIXTURE=.../chough/examples/shipping`). Both binaries
gave the same result:

```text
== tracked files holding U+2014 or U+2013 before review/finalize:
(none)
== tracked files holding U+2014 or U+2013 after finalize:
.specsync/archive/changes/2026-11-01-free-shipping-at-the-threshold/lesson-bundle.md:1:# Lesson bundle — free-shipping-at-the-threshold
== hand-edit the archived heading to a colon, commit, then audit and check
change audit exit=0
check --strict exit=0
== part 2: agent instruction files from `specsync hooks install` in an empty repo
AGENTS.md: 18 lines with U+2014
CLAUDE.md: 18 lines with U+2014
.specsync/.gitignore: 1 lines with U+2014
```

A second script covered the writers the issue found only by byte grep. It used the origin/main
build in an empty Git repository with `src/lib.rs`:

1. `specsync init`, `specsync hooks install` (all targets), `specsync agents install` (all tools)
   and `specsync add-spec widget`.
2. Twelve rows under the spec's `## Change Log` (a 3-column table, then a 2-column one), then
   `specsync compact --keep 3`.
3. In a separate directory holding a 3.x `specsync.json`, `specsync migrate`.

Lines holding U+2014 per written file:

| Writer | File | Lines with U+2014 |
|--------|------|-------------------|
| `hooks install --claude` | `CLAUDE.md` | 18 |
| `hooks install --agents` | `AGENTS.md` | 18 |
| `hooks install --copilot` | `.github/copilot-instructions.md` | 8 |
| `hooks install --cursor` | `.cursorrules` | 7 |
| `hooks install --precommit` | `.git/hooks/pre-commit` | 5 |
| `agents install --claude` | `.claude/commands/specsync/create-spec.md` | 4 |
| `agents install --cursor` | `.cursor/commands/specsync-create-spec.md` | 4 |
| `init` | `.specsync/.gitignore` | 1 (`# spec-sync 5.0`, dash, `generated by ...`) |
| `migrate` | `.specsync/.gitignore` | 1 (`# spec-sync v4`, dash, `generated by ...`) |
| `compact` | the spec's Change Log | 1 per summary row |
| `change finalize` | archive `lesson-bundle.md` | 1 (the heading) |

The compaction rows observed:

```text
| 2026-01-01 — 2026-01-09 | — | Compacted: 9 entries <!-- specsync:compact:v1 --> |
| 2026-02-01 — 2026-02-09 | Compacted: 9 entries <!-- specsync:compact:v1 --> |
```

`add-spec`, `.claude/settings.json`, the Gemini command TOML and every generated `SKILL.md` wrote
none.

Hand-editing the archived lesson-bundle heading to a colon passes `change audit` and
`check --strict`. That matches the code: nothing reads the bundle back (see below).

## Root cause

Each writer embeds the dash in a string literal. There is no shared template layer and no test
over written bytes, so nothing stopped them.

- `src/change/historical_definition_digest_matches.rs:2823`, `write_lesson_bundle`, formats the
  heading with the dash. The bundle is written after the archive move
  (`let _ = write_lesson_bundle(...)` at line 2422) and is never parsed or digested again.
- `src/hooks.rs:9`, `:57`, `:83` and `:108`: `CLAUDE_MD_SNIPPET`, `CURSORRULES_SNIPPET`,
  `COPILOT_INSTRUCTIONS_SNIPPET` and `AGENTS_MD_SNIPPET` use the dash as a definition separator
  and as sentence punctuation.
- `src/hooks.rs:511-512`, `HOOK_BEGIN`, carries the dash. It is not only output.
  `remove_section_from_file` (`src/hooks.rs:827`) finds the block with `l.contains(HOOK_BEGIN)`.
  Every block installed by a sentinel-era version (since #300, 1f1c3a44) up to 6.1.1 carries
  this exact line.
- `src/hooks.rs:156-180`, `PRE_COMMIT_HOOK`: comment lines and one `echo` carry the dash.
  `pre_commit_block` (line 276) derives the managed block body from it. The constant is also
  compared by `uninstall_precommit` for unmarked legacy hooks: the lines after
  `LEGACY_PRE_COMMIT_MARKER` (line 671) and a whole-file hook (line 679). Those comparisons do
  not protect any state a release wrote with the current text (see the next section).
- `src/commands/init.rs:565`, `specsync_gitignore_content`, and `src/commands/migrate.rs:682`,
  `apply_write_gitignore`: header comment lines. Both check only that the file exists before
  writing, and nothing parses the header.
- `src/agents.rs:128`, `:130`, `:136` and `:156`: `CREATE_SPEC_STEPS_MD`, shared by the Claude and
  Cursor create-spec commands. Installed files are tracked by digest in
  `.specsync/agent-artifacts.json`, with `AGENT_ARTIFACT_TEMPLATE_VERSION = 5` (line 227).
- `src/compact.rs:313` and `:315` write the summary row's range separator and placeholder cells.
  `summary_metadata` (`src/compact.rs:500` and `:503`) is the reader. A prior summary is
  recognized only when every interior cell equals the dash and the first cell splits on a
  space-dash-space separator.

## What a writer-only fix breaks (read from the code, not built)

- **`HOOK_BEGIN` alone.** `hooks uninstall` on a 6.1.1 block finds no begin sentinel. It does not
  match the exact current snippet either, because the snippet text changes too. It falls back to
  the heuristic, which starts at the `# Spec-Sync Integration` line. The old begin comment is left
  behind as an orphan line in the user's file.
- **`PRE_COMMIT_HOOK` alone breaks nothing a release wrote.** This was checked against history,
  not assumed:
  - Only releases before #480 (32c8b350, 2026-07-31, first in v6.0.0-rc.1) wrote the unmarked
    forms: `LEGACY_PRE_COMMIT_MARKER` followed by the body, or the body as the whole file
    (`git show v5.2.0:src/hooks.rs`, `install_precommit`).
  - #774 (b9ff3231, 2026-09-09) then rewrote `PRE_COMMIT_HOOK`: the config path moved from
    `specsync.json` to `.specsync/config.toml`, the `(default)` moved from `warn` to `strict`, and
    the `--enforcement` comment became the two `--strict` lines.
  - So since #774 the comparisons at lines 671 and 679 already fail for every real legacy
    install, whose body is the v5.x text. The 6.1.1 body only ever appeared inside project-keyed
    blocks, which are found by markers that carry no dash.
  - Repunctuating `PRE_COMMIT_HOOK` therefore needs no frozen copy. An earlier version of this
    draft proposed one (`LEGACY_PRE_COMMIT_HOOK` holding the 6.1.1 text). That protected a state
    no release produced, and it contradicted `research.md` alternative 3, which turns down
    frozen snippet copies for the same reason. It is dropped.
  - The real gap, that `hooks uninstall --precommit` stopped recognizing v5.x unmarked hooks at
    #774, predates this issue and is not fixed here. It is listed as an open question in
    `tasks.md`.
- **The compaction writer alone.** Every changelog compacted by 6.1.1 would read as user history
  on the next run. That run would emit a second marked summary, and the run after it would fail
  closed on duplicate marked summaries.

## Out of scope, and why

- Terminal, stderr and JSON text (`Handoff: safe`, dash, ..., warnings, `next_action` strings),
  and Markdown printed to stdout:
  - `compact` and `archive-tasks` with `--format markdown`,
  - `specsync comment` bodies,
  - `issues` verification.
  The issue scopes these out because they are not written into the repository. About 200 such
  literals remain.
- User text that the lesson bundle copies verbatim. That is not only the `context.md`,
  `design.md` and `testing.md` bodies. `write_lesson_bundle`
  (`src/change/historical_definition_digest_matches.rs:2821-2899`) also copies `record.title`,
  every acceptance criterion, `affected_specs`, `affected_paths` and the recorded verification
  command strings word for word. A dash in any of them is the user's and stays.
- Recognition of v5.x unmarked pre-commit hooks, lost at #774 (see above).
- Frozen archives under `.specsync/archive/`, and blocks or rows already present in a user's
  files. `hooks install` is a no-op when a block exists (`specs/hooks/requirements.md` Out of
  Scope: no auto-update on upgrade). Compaction rewrites a summary only when it folds more rows.
- SpecSync's own hand-written docs, specs, source comments and this repository's `AGENTS.md`.

## Process note

The shared scratch directory used while drafting is also used by sibling agents. One
`change answer` read a criteria file that another agent had overwritten, which put two foreign
criteria into this record. The answer was re-recorded from a private file, and `change.md` now
holds only this change's criteria. Nothing else was affected.

## Coordination with sibling drafts

- Requirement ids. Origin/main ends at REQ-change-112. The draft for #826 adds REQ-change-113,
  and the draft for #825 adds REQ-change-114 and REQ-change-115. This package takes
  REQ-change-116 so that no two drafts claim the same id, whatever order they are approved in.
- Merge conflict. The draft for #825 also edits
  `src/change/historical_definition_digest_matches.rs` (4,412 lines). This change touches only
  the heading line in `write_lesson_bundle`, so the conflict, if any, is local. Whichever lands
  second rebases.

## From the change's design.md

# Design

One rule: **writers emit the new form, and readers accept the new form and the 6.1.1 form
wherever a release actually wrote the 6.1.1 form.**
No existing file is rewritten by upgrading. Punctuation follows the table in `research.md`.

## Part A: write-only generators (no reader to keep)

- `write_lesson_bundle` (`src/change/historical_definition_digest_matches.rs:2823`) writes
  `# Lesson bundle: <id>`. Nothing else in the bundle's fixed text carries a dash. Every piece
  of user-supplied record text it interpolates is left byte for byte: `record.title`, each
  acceptance criterion, `affected_specs`, `affected_paths`, the recorded verification command
  strings, and the `context.md`, `design.md` and `testing.md` bodies.
- `specsync_gitignore_content` (`src/commands/init.rs:565`) writes
  ``# spec-sync 5.0: generated by `specsync init` ``.
- `apply_write_gitignore` (`src/commands/migrate.rs:682`) writes
  ``# spec-sync v4: generated by `specsync migrate` ``.
- `CREATE_SPEC_STEPS_MD` (`src/agents.rs:123`) has four dashes. The two after bold
  definitions (lines 128 and 130) become colons, the aside on line 136 becomes a semicolon, and
  the lead-in on line 156 becomes a colon.
  - `AGENT_ARTIFACT_TEMPLATE_VERSION` goes from 6 to 7.
  - The existing digest manifest refreshes an installed copy whose bytes still match the
    recorded digest, and reports a customized copy as a conflict, as today. No manifest logic
    changes.

## Part B: hook instruction snippets and the begin sentinel

- The four snippets (`src/hooks.rs:9`, `:57`, `:83`, `:108`) are rewritten with colons, commas
  and semicolons. No instruction is added, removed or reworded beyond punctuation. Each heading
  stays exactly as it is, so the `Spec-Sync Integration` and `Spec-Sync Rules` detection keeps
  working.
- Sentinels:

  ```rust
  const HOOK_BEGIN: &str =
      "<!-- specsync:hook:begin (managed by `specsync hooks`; do not edit inside) -->";
  /// The begin sentinel written by sentinel-era releases up to and including 6.1.1.
  /// Read, never written.
  const LEGACY_HOOK_BEGIN: &str =
      "<!-- specsync:hook:begin (managed by `specsync hooks` \u{2014} do not edit inside) -->";
  const HOOK_END: &str = "<!-- specsync:hook:end -->"; // unchanged
  fn is_hook_begin(line: &str) -> bool {
      line.contains(HOOK_BEGIN) || line.contains(LEGACY_HOOK_BEGIN)
  }
  ```

  - The legacy literal is written with the `\u{2014}` escape, so `src/hooks.rs` carries no raw
    dash in any string it writes and a byte grep of the source stays a useful check.
  - `remove_section_from_file` uses `is_hook_begin`. Strategies 2 and 3 are unchanged.
- Install and status need no change: both detect by heading text.

## Part C: pre-commit hook body

- `PRE_COMMIT_HOOK` is rewritten:
  - The first comment becomes `# spec-sync pre-commit hook: validates specs before allowing commits.`
  - The three enforcement lines become `#   "warn":         report ...` (alignment kept).
  - The failure echo becomes `specsync: specs have errors; fix them before committing.`
  - Both `replacen` anchors in `pre_commit_block` (`if ! specsync check; then` and the
    `Run 'specsync check'` echo) carry no dash and are untouched.
- No reader changes. Every release from #480 on writes the body inside project-keyed markers,
  which carry no dash, so `uninstall_precommit` finds and removes a 6.1.1 block by its markers
  whatever body it holds.
- The unmarked legacy comparisons (`src/hooks.rs:671` and `:679`) keep comparing against
  `PRE_COMMIT_HOOK`. No release wrote an unmarked hook with the 6.1.1 text, and the v5.x text
  that releases did write stopped matching at #774 (see `context.md`). No frozen copy is added.
  Restoring v5.x recognition is a separate question in `tasks.md`.

## Part D: compaction summary rows

- Writer (`src/compact.rs:313-315`) emits `{first} to {last}` and `-` placeholders:

  ```text
  | 2026-01-01 to 2026-01-09 | - | Compacted: 9 entries <!-- specsync:compact:v1 --> |
  ```

- Reader (`summary_metadata`):
  - Interior cells must be uniformly `-` or uniformly the legacy dash. A mix is not a summary.
  - The range splits on the first ` to `, or else on the legacy space-dash-space separator.
  - The count, label and `<!-- specsync:compact:v1 -->` marker rules are unchanged.
- Behavior:
  - A no-excess run still returns the original bytes, so a legacy-form row is left as it is.
  - A folding run replaces the prior summary, legacy or new, with one new-form row. It keeps the
    original range start and adds the counts.
  - A duplicate marked summary in any mix of forms fails closed, as today.

## Part E: the guard test

- A new integration test, `tests/integration/generated_output_has_no_dashes.rs`, covers the
  writers in the second acceptance criterion only. It runs in a disposable Git repository
  through the built binary:
  - `init`, `hooks install`, `agents install`, `add-spec`, and `compact` on a 3-column and a
    2-column changelog;
  - `migrate` on `helpers::setup_v3_project`.
  It walks every file the binary wrote, including `.git/hooks/pre-commit` and excluding the rest
  of `.git/`. It fails naming each file and line holding U+2013 or U+2014.
- Each module gets a unit test that renders its templates in-process and asserts the same.
  These are for fast local feedback and name the constant that regressed.
- The lesson bundle is covered only in-process, by unit tests that call `write_lesson_bundle`.
  A binary-driven finalize needs a fully approved, checked and reviewed change, which this test
  cannot produce honestly. The acceptance criterion says so. One test asserts the fixed text adds
  no dash; another puts U+2014 in the title, a criterion and `context.md` and asserts each is
  copied unchanged. Details are in `testing.md`.

## Resulting behavior

| Situation | origin/main a80748ff | After |
|-----------|----------------------|-------|
| Fresh `hooks install` / `agents install` / `init` / `migrate` | U+2014 in 9 written files | none |
| `change finalize` | bundle heading has U+2014 | `# Lesson bundle: <id>` |
| `compact` writes a summary | dash range and placeholders | ` to ` and `-` |
| `hooks uninstall` on a 6.1.1 block | removes it exactly | removes it exactly |
| `hooks uninstall --precommit` on a 6.1.1 project-keyed block | removes it | removes it |
| `hooks uninstall --precommit` on a v5.x unmarked hook | not recognized (since #774) | not recognized (unchanged) |
| `compact` re-run on a 6.1.1-compacted changelog, no excess | unchanged bytes | unchanged bytes |
| `compact` folding into a 6.1.1 summary | one dash-form row | one new-form row, same count and start |
| Existing installed files and archives | as written | untouched |

## Compatibility and contract notes

- **Observable file-format changes**:
  - the hook begin sentinel;
  - the pre-commit hook body text (comments and one `echo`; no reader depends on it);
  - the compaction summary row grammar (REQ-compact-001 is modified);
  - the lesson-bundle heading;
  - the two `.gitignore` headers;
  - the create-spec command text;
  - `template_version` 6 in `.specsync/agent-artifacts.json`.
- No CLI flag, JSON field, exit code or policy schema changes.
- **Downgrade.** An older binary reading new output:
  - It does not recognize the new hook begin sentinel. Uninstall falls to the exact-snippet
    strategy, which fails because the text changed, then to the heuristic, which leaves the new
    begin line behind.
  - It does not recognize a new-form compaction row as a summary, so it treats the row as the
    oldest ordinary entry. A folding run then compacts it like any entry: its accumulated count
    is dropped, and its whole first cell (`<first> to <last>`) becomes the start of the new
    range. Nothing fails, but the history count is wrong.
  - Both are one-way format changes and are stated in the release notes (see `docs.md`).

## From the change's testing.md

# Testing

Each new test carries an honest label:

- A DISCRIMINATOR fails on origin/main a80748ff and passes with the fix.
- A CONTROL passes on both. Controls guard the compatibility promise: on origin/main the legacy
  form is the only form, so a legacy-reading test cannot fail there.

Labels are confirmed against a build of unfixed a80748ff from a separate checkout before review.
Results are recorded here after verification, not assumed.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-hooks-005` | `generated_hook_content_contains_no_em_or_en_dash`, `uninstall_removes_a_block_with_the_new_begin_sentinel_exactly`, `uninstall_removes_a_block_with_the_6_1_1_begin_sentinel_exactly`, `uninstall_precommit_removes_a_managed_block_with_the_6_1_1_body` (src/hooks.rs); `fresh_project_writers_emit_no_em_or_en_dash` and `hooks_uninstall_through_the_binary_removes_a_6_1_1_block` (integration). |
| `REQ-agents-010` | Criteria 1 and 2: `generated_agent_artifacts_contain_no_em_or_en_dash`, `manifest_allows_unmodified_generated_artifact_upgrade` (src/agents.rs); `fresh_project_writers_emit_no_em_or_en_dash` (integration). Criterion 3: `tracked_agent_artifacts_and_manifest_match_a_fresh_install` (src/agents.rs), added with the regeneration task in `tasks.md`; it failed before the regeneration and passes after it (see "SKILL.md regeneration" below), so criterion 3 is met. At bb7b4b4d criterion 3 was NOT met: `checked_in_create_spec_commands_match_generated_assets` pins the create-spec commands, and every command asset and command manifest entry matches a fresh install, but the four `*/skills/spec-sync/SKILL.md` manifest entries (template 5, digest 43c72636...) and files (e0db91e7...) differ from the generated ones (template 6, digest 94d1a150...). At bb7b4b4d those files were outside the approved `affected_paths`; 0xLeif's amendment adds them, and the regeneration task in `tasks.md` closes the gap. |
| `REQ-compact-001` | `compact_summary_rows_contain_no_em_or_en_dash`, `compact_leaves_a_6_1_1_summary_byte_for_byte_without_excess`, `compact_folds_a_6_1_1_summary_into_one_new_form_row`, `compact_does_not_treat_mixed_placeholder_forms_as_a_summary`, `compact_does_not_treat_a_range_and_placeholder_form_hybrid_as_a_summary`, `compact_refuses_a_range_start_that_contains_the_separator`, `compact_folds_when_only_the_range_end_contains_the_separator` and the existing compact suite (src/compact.rs); `fresh_project_writers_emit_no_em_or_en_dash` (integration). |
| `REQ-change-116` | `lesson_bundle_heading_uses_a_colon_and_adds_no_dash`, `lesson_bundle_copies_user_record_text_verbatim` (`lesson_bundle_dash_tests` in src/change/historical_definition_digest_matches.rs). |
| `REQ-cmd-init-008` | `specsync_gitignore_content_has_no_em_or_en_dash` (src/commands/init.rs); `fresh_project_writers_emit_no_em_or_en_dash` (integration). |
| `REQ-cmd-migrate-004` | `apply_write_gitignore_writes_no_em_or_en_dash` (src/commands/migrate.rs); `migrate_writes_no_em_or_en_dash` (integration). |

## Integration: `tests/integration/generated_output_has_no_dashes.rs`

New module, registered in `tests/integration.rs`. It covers the writers named in the second
acceptance criterion (init, migrate, hooks install, agents install, compact) and nothing else.
It drives the built binary through
`helpers::specsync()` in a `TempDir` that is `git init`ed. The shared assertion walks every file
under the root, including `.git/hooks/pre-commit` and excluding the rest of `.git/`. It fails
listing each `path:line` that holds U+2013 or U+2014.

- DISCRIMINATOR `fresh_project_writers_emit_no_em_or_en_dash`:
  - writes `src/lib.rs`;
  - runs `init`, `hooks install`, `agents install` and `add-spec widget`;
  - appends twelve rows to a 3-column and to a 2-column `## Change Log`, then runs
    `compact --keep 3`.
  Every written file passes. The test also asserts that the summary rows read
  `| 2026-01-01 to 2026-01-09 | - | Compacted: 9 entries <!-- specsync:compact:v1 --> |` and the
  2-column equivalent.
- DISCRIMINATOR `migrate_writes_no_em_or_en_dash`: `helpers::setup_v3_project` then `migrate`.
  `.specsync/.gitignore` starts with ``# spec-sync v4: generated by `specsync migrate` ``.
- CONTROL `hooks_uninstall_through_the_binary_removes_a_6_1_1_block`:
  - writes `CLAUDE.md` and `AGENTS.md`, each holding user text, then a block framed by the
    6.1.1 begin line and the end line, then more user text;
  - runs `hooks uninstall --claude --agents`.
  Both files keep exactly the user text, and no `specsync:hook` line remains.

## Unit tests

`src/hooks.rs` tests module:

- DISCRIMINATOR `generated_hook_content_contains_no_em_or_en_dash`. The four `wrap_snippet`
  outputs, `PRE_COMMIT_HOOK`, `pre_commit_block` for a `setup_git` root, and
  `CLAUDE_CODE_HOOK_SETTINGS` contain neither character.
- DISCRIMINATOR `uninstall_removes_a_block_with_the_new_begin_sentinel_exactly`. The fixture
  is user text, then a block framed by the new begin line written as a string literal (not via
  `HOOK_BEGIN`), then user text. Uninstall leaves both user parts and no orphan line. On
  origin/main the begin line is not recognized and the heuristic leaves it behind.
- CONTROL `uninstall_removes_a_block_with_the_6_1_1_begin_sentinel_exactly`. The same fixture,
  framed by the frozen 6.1.1 begin line written out as a literal, with a 6.1.1-punctuated body.
- CONTROL `uninstall_precommit_removes_a_managed_block_with_the_6_1_1_body`: a project-keyed
  block, framed by the markers `pre_commit_markers` returns for the root, holding the 6.1.1 body
  written out as a literal (this is the form every release from #480 to 6.1.1 wrote). Uninstall
  removes the block and leaves user lines around it.
- No test is added for an unmarked hook with the 6.1.1 body: no release wrote one (see
  `context.md`).
- Existing tests kept green unchanged: `uninstall_precommit_removes_legacy_appended_block` and
  `uninstall_precommit_removes_legacy_whole_file_hook`, which build their fixtures from
  `PRE_COMMIT_HOOK` and so exercise the new body, as they exercised the current body before;
  `uninstall_preserves_user_content_after_wrapped_block`;
  `uninstall_legacy_block_without_sentinels_preserves_appended_notes`;
  `uninstall_preserves_crlf_line_endings`; and
  `install_then_uninstall_roundtrip_preserves_appended_notes`.

`src/compact.rs` tests module:

- DISCRIMINATOR `compact_summary_rows_contain_no_em_or_en_dash`, for 2-, 3- and 4-column
  tables.
- CONTROL `compact_leaves_a_6_1_1_summary_byte_for_byte_without_excess`, with a legacy-form
  summary and LF and CRLF variants.
- DISCRIMINATOR `compact_folds_a_6_1_1_summary_into_one_new_form_row`. A legacy
  `Compacted: 9 entries` summary plus excess rows yields one new-form row. The row keeps the
  legacy range start, sums the counts, and carries exactly one marker.
- CONTROL `compact_does_not_treat_mixed_placeholder_forms_as_a_summary`. A 4-column row with
  one `-` and one legacy-dash placeholder plus the marker stays user history. It is not a summary
  on origin/main either; the test stops the new reader from widening to it.
- CONTROL `compact_does_not_treat_a_range_and_placeholder_form_hybrid_as_a_summary` (review
  round). On 3- and 4-column tables, a U+2014 range with `-` interiors and a ` to ` range with
  U+2014 interiors are both user history: they match neither approved form.
- DISCRIMINATOR `compact_refuses_a_range_start_that_contains_the_separator` (review round).
  First cells such as `Q1 to Q2 2026` would be written as an ambiguous range whose start the
  next fold cuts short, so compaction refuses and writes nothing.
- CONTROL `compact_folds_when_only_the_range_end_contains_the_separator` (review round). A
  range end holding ` to ` stays unambiguous and folds with the full original start.
- Existing tests whose expected strings move to the new form:
  - `compact_is_idempotent`;
  - `compact_folds_existing_summary_when_more_entries_arrive`;
  - `compact_summary_row_matches_four_column_table`;
  - `compact_reports_kept_entries_not_summary_row`.
  `compact_requires_summary_placeholders_in_wide_tables` and
  `compact_does_not_treat_user_text_as_a_summary_row` keep their legacy-form user rows,
  because they test that unmarked rows stay user history.

`src/agents.rs` tests module:

- DISCRIMINATOR `generated_agent_artifacts_contain_no_em_or_en_dash`, for every tool through
  `install_agent`.
- Updated: the template-version upgrade test (around line 1455, using
  `AGENT_ARTIFACT_TEMPLATE_VERSION - 1`) passes at version 7.
- DISCRIMINATOR (against the pre-regeneration head bb7b4b4d)
  `tracked_agent_artifacts_and_manifest_match_a_fresh_install`. It runs `install_agent` for
  every tool (Claude, Codex, Cursor, Gemini) into a fresh temporary root, then compares that
  root with this repository: every generated file, including the four
  `*/skills/spec-sync/SKILL.md` files, must be byte-identical to the tracked copy, and the
  fresh `.specsync/agent-artifacts.json` entries (path, tool, `template_version`, digest) must
  equal the tracked manifest entry for entry, with no tracked entry missing or extra. It fails
  at bb7b4b4d, naming the four `SKILL.md` files and their manifest entries (template 5, digest
  43c72636... against template 6, digest 94d1a150...), and passes after the regeneration. It
  also fails on origin/main a80748ff, because the `SKILL.md` drift predates this change (#808).
- Kept green after regenerating the tracked copies with the built binary:
  `checked_in_create_spec_commands_match_generated_assets`,
  `tracked_skill_files_carry_the_current_same_actor_guidance` and
  `tracked_skill_files_teach_check_is_the_product`.

`src/change_tests/canonical_resolution_tolerates_inert_legacy_registry_stub.rs` (4,020 lines,
under the REQ-change-109 limit), next to
`a_module_3md_file_shows_later_planes_as_lessons_and_leaves_context_md`:

- DISCRIMINATOR `lesson_bundle_heading_uses_a_colon_and_adds_no_dash`. `write_lesson_bundle`
  for a record with a title, two acceptance criteria, affected specs and paths, a recorded
  verification command, and `context.md`, `design.md` and `testing.md` bodies, none holding a
  dash, yields a first line `# Lesson bundle: <id>` and no U+2013 or U+2014 anywhere.
- DISCRIMINATOR `lesson_bundle_copies_user_record_text_verbatim`. U+2014 is placed in the
  title, in one acceptance criterion, in an affected path and in the `context.md` body. Each
  appears in the bundle exactly as written, and the count of U+2014 in the bundle equals the
  count in those inputs, so the bundle adds none. On origin/main the copy assertions pass and
  the count assertion fails by one, because the heading adds a dash.
- This is the only coverage of the lesson bundle. A finalize through the built binary needs a
  fully approved, checked and reviewed change, so the integration test does not attempt one.

`src/commands/init.rs` and `src/commands/migrate.rs` tests modules:

- DISCRIMINATOR `specsync_gitignore_content_has_no_em_or_en_dash`.
- DISCRIMINATOR `apply_write_gitignore_writes_no_em_or_en_dash`.

## Regression lanes

1. `cargo test --lib hooks:: compact:: agents:: commands::init commands::migrate`
2. `cargo test --lib change_tests::` (lesson bundle and the change module)
3. `cargo test --test integration generated_output_has_no_dashes`
4. `cargo test --test integration` (full integration suite, including `quickstart.rs` and
   `commands/specsync_process.rs`, which drive `hooks install`)
5. `specsync check --strict --require-coverage 100 --force`
6. `fledge lanes run pre-push`, then `fledge lanes run verify` once the PR is merge-ready

The issue's `repro3.sh` and the extra-writers script from `context.md` are re-run with the fixed
build. Their output is recorded here.

## Results (recorded after verification, 2026-11-01)

Fail-before: the new tests were added first and run against the unchanged tree (a80748ff plus
the approval commits). Every DISCRIMINATOR failed and every CONTROL passed:

- Unit: `generated_hook_content_contains_no_em_or_en_dash`,
  `uninstall_removes_a_block_with_the_new_begin_sentinel_exactly` (the new begin line was left
  behind as an orphan), `compact_summary_rows_contain_no_em_or_en_dash`,
  `compact_folds_a_6_1_1_summary_into_one_new_form_row`,
  `generated_agent_artifacts_contain_no_em_or_en_dash`,
  `lesson_bundle_heading_uses_a_colon_and_adds_no_dash`,
  `lesson_bundle_copies_user_record_text_verbatim` (6 em dashes, expected 5),
  `specsync_gitignore_content_has_no_em_or_en_dash` and
  `apply_write_gitignore_writes_no_em_or_en_dash` failed, as did the five existing compact tests
  whose expected strings moved to the new form. The controls
  `uninstall_removes_a_block_with_the_6_1_1_begin_sentinel_exactly`,
  `uninstall_precommit_removes_a_managed_block_with_the_6_1_1_body`,
  `compact_leaves_a_6_1_1_summary_byte_for_byte_without_excess` and
  `compact_does_not_treat_mixed_placeholder_forms_as_a_summary` passed.
- Integration: `fresh_project_writers_emit_no_em_or_en_dash` failed listing 67 lines (CLAUDE.md 18,
  AGENTS.md 18, copilot-instructions 8, .cursorrules 7, pre-commit 5, the two create-spec
  commands 4 each, `.specsync/.gitignore` 1, one summary row per spec);
  `migrate_writes_no_em_or_en_dash` failed on the gitignore header;
  `hooks_uninstall_through_the_binary_removes_a_6_1_1_block` passed.

Pass-after, with the fix:

- `cargo test --bin specsync`: 2636 passed, 0 failed (after regenerating the two tracked
  create-spec commands, which `checked_in_create_spec_commands_match_generated_assets` pins).
- `cargo test --test integration`: 467 passed, 0 failed.
- `specsync check --strict --require-coverage 100 --force` (release build): 62 specs passed,
  0 warnings, 129/129 files.
- The extra-writers script from `context.md`, re-run with the fixed release build: no written
  file outside `.git/` and no line of `.git/hooks/pre-commit` holds U+2013 or U+2014; the
  summary row reads `| 2026-01-01 to 2026-01-09 | - | Compacted: 9 entries <!-- specsync:compact:v1 --> |`;
  `hooks uninstall` removed every block it installed.
- The lesson bundle was not re-run through a binary finalize, as stated above; the two
  in-process tests are its coverage.

Deviation on placement: the two lesson-bundle tests were first written in
`src/change_tests/canonical_resolution_tolerates_inert_legacy_registry_stub.rs`, as planned above.
That file is outside this change's approved `affected_paths`, and hosted CI's lifecycle audit
refused it ("meaningful changed paths are not covered by an active change"). The tests moved,
unchanged in substance, into a `#[cfg(test)] mod lesson_bundle_dash_tests` at the end of
`src/change/historical_definition_digest_matches.rs` (4,530 lines, under the REQ-change-109
limit), beside `write_lesson_bundle`. Re-run against the old heading they fail (the count test
reports 6 em dashes, expected 5); with the fix both pass.

## Review round on PR #829 (2026-11-01)

Fail-before, with the three tests above added to the unchanged a646691f code:
`compact_does_not_treat_a_range_and_placeholder_form_hybrid_as_a_summary` failed (the U+2014
range with `-` interiors was read as a summary) and
`compact_refuses_a_range_start_that_contains_the_separator` failed (compaction wrote
`Q1 to Q2 2026 to Q5 to Q6 2026`). `compact_folds_when_only_the_range_end_contains_the_separator`
passed. Pass-after: `cargo test --bin specsync compact`, 41 passed, 0 failed.
`compact_does_not_treat_mixed_placeholder_forms_as_a_summary` now starts its table with a dated
row, because its mixed row's ` to ` range would otherwise become the refused range start.

REQ-agents-010 criterion 3 is not met at this head, as recorded in the evidence table. The
builder cannot regenerate the four SKILL.md files: they are outside the approved
`affected_paths`, and the hosted lifecycle audit refuses changed paths no active change covers.
0xLeif decided to add the four paths to `affected_paths` and regenerate. After re-approval the
regeneration and `tracked_agent_artifacts_and_manifest_match_a_fresh_install` are run, and
their results are recorded here.

## SKILL.md regeneration under the re-approved scope (2026-11-01)

0xLeif re-approved the amended definition (four `*/skills/spec-sync/SKILL.md` paths added to
`affected_paths`); `specsync change status <id> --json` reported `approval_valid: true` before
any of this work.

- Fail-before: `tracked_agent_artifacts_and_manifest_match_a_fresh_install` added to
  `src/agents.rs` and run against the unregenerated tracked files
  (`cargo test --locked --bin specsync tracked_agent_artifacts_and_manifest_match_a_fresh_install`):
  FAILED, naming exactly eight drifts. Each of the four `SKILL.md` files differs (tracked
  e0db91e7..., fresh 94d1a150...), and each of their four manifest entries differs (tracked
  template 5 digest 43c72636..., fresh template 6 digest 94d1a150...). No command asset or
  command entry was reported.
- Regeneration: `cargo build --locked --release`, then
  `target/release/specsync agents install --root <empty scratch dir>` (all four tools), then the
  scratch root's four `SKILL.md` files and `.specsync/agent-artifacts.json` copied over the
  tracked ones. `agents install` has no force flag and refuses the drifted files in place, so a
  fresh install copied back is the exact output a fresh install writes. After the copy, every
  one of the 17 files in the scratch root is byte-identical (`cmp`) to the tracked file.
- Content change: one line per `SKILL.md`, the `context.md` bullet gains the `<module>.3md`
  lesson-source sentence from the current `SKILL_BODY`. All four files now hash 94d1a150...;
  the manifest's four `SKILL.md` entries are template 6, digest 94d1a150....
- Dash check: U+2014 and U+2013 occur 0 times in each regenerated `SKILL.md` and in
  `.specsync/agent-artifacts.json`.
- Pass-after: `tracked_agent_artifacts_and_manifest_match_a_fresh_install` passes;
  `cargo test --locked --bin specsync agents::`, 44 passed, 0 failed (including
  `checked_in_create_spec_commands_match_generated_assets`,
  `tracked_skill_files_carry_the_current_same_actor_guidance` and
  `tracked_skill_files_teach_check_is_the_product`).

## Where these lessons go

- `specs/hooks/hooks.3md`
- `specs/agents/agents.3md`
- `specs/compact/compact.3md`
- `specs/change/change.3md`
- `specs/cmd_init/cmd_init.3md`
- `specs/cmd_migrate/cmd_migrate.3md`
