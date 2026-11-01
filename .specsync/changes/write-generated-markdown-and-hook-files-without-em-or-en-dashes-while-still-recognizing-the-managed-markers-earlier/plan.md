---
change: write-generated-markdown-and-hook-files-without-em-or-en-dashes-while-still-recognizing-the-managed-markers-earlier
artifact: plan
---

# Plan

## Approach

Replace U+2014 in every literal that SpecSync writes into a repository file with a colon, comma
or semicolon. For the two strings that are read back in a form a release actually wrote (the hook
begin sentinel and the compaction summary row), readers accept the 6.1.1 form too. The
pre-commit hook body needs no frozen copy: since #480 it is written only inside project-keyed
markers that carry no dash (see `context.md`). Add one integration test over the bytes the binary writes, plus per-module unit tests,
so a new dash in any writer fails CI.

## Out of scope

- Terminal, stderr, JSON and stdout Markdown text, including `--format markdown` reports,
  `specsync comment` bodies and issue verification output.
- Rewriting existing installed blocks, compacted rows or frozen archives.
- Auto-refreshing a present hook block on `hooks install`.
- Text the lesson bundle copies from user artifacts.
- SpecSync's own hand-written docs, specs and source comments.
- Capturing the proposed human-intent criteria. That is the human's decision.
- Release, tag or merge.

## Steps

1. Write the discriminator and control tests in `testing.md`, and confirm their labels against
   an unfixed a80748ff build.
2. Part A: the lesson-bundle heading, the `init` and `migrate` `.gitignore` headers, and
   `CREATE_SPEC_STEPS_MD`. Bump `AGENT_ARTIFACT_TEMPLATE_VERSION` to 6.
3. Part B: rewrite the four hook snippets, add `LEGACY_HOOK_BEGIN` and `is_hook_begin`, and use
   them in `remove_section_from_file`.
4. Part C: rewrite `PRE_COMMIT_HOOK`. `uninstall_precommit` is unchanged.
5. Part D: change the compaction writer, and teach `summary_metadata` both forms.
6. Regenerate this repository's agent assets with the built binary
   (`target/release/specsync agents install`). Commit the changed
   `.claude/commands/specsync/create-spec.md`, `.cursor/commands/specsync-create-spec.md`,
   the four `*/skills/spec-sync/SKILL.md` files (`.claude`, `.codex`, `.cursor`, `.gemini`,
   added to `affected_paths` by 0xLeif's amendment on PR #829) and
   `.specsync/agent-artifacts.json`. `tracked_agent_artifacts_and_manifest_match_a_fresh_install`
   pins the result.
7. Update the companions after verification:
   - `specs/compact/requirements.md`: the free acceptance list that still names the dash form;
   - `compact.3md` notes;
   - lessons on a later plane of `hooks.3md`, `compact.3md` and `agents.3md`;
  - the new requirement in `specs/change/requirements.md` as REQ-change-116;
   - each touched spec's version and Change Log.
8. Run the regression lanes in `testing.md`. Then run `fledge lanes run pre-push` before any
   push.

## Risks

- **Downgrade is one-way.** An older binary does not recognize the new begin sentinel or the new
  summary row (see `design.md`). This is stated in the PR so the release notes can carry it.
- **Missing a reader.** A writer changed without its reader would strand existing installs. The
  `research.md` table lists every string read back. The controls in `testing.md` pin each legacy
  form as a literal fixture, so the test does not follow a constant that changes.
- **Missing a writer.** The integration test walks every written byte rather than a list of
  constants. A writer this change does not exercise, such as `scaffold`, `generate`, `import` or
  `init-registry`, is covered by a byte grep of the string literals (none found) but not by the
  test. The open question in `tasks.md` asks whether to extend it.
- **Sibling drafts.** #825 and #826 add requirement ids in the same module, and #825 edits the
  same change source file. The id is chosen to avoid a collision, and the source edit here is
  one line.
- **Range separator.** A changelog whose first cell contains ` to ` would truncate the recorded
  range start. Generated and documented first cells are dates.

## Constraints consulted

- `specs/hooks/hooks.3md` and REQ-hooks-002: detection by heading markers, and exact
  block-only uninstall that preserves user and other-project content.
- `specs/compact/compact.3md` and REQ-compact-001: provenance-marked ownership, byte-for-byte
  idempotence, fold-forward of count and range start, and fail-closed on duplicates.
- `specs/agents/agents.3md` and REQ-agents-007/008: tracked assets are installer output,
  parity-tested byte for byte, and the template version advances when content changes.
- `specs/change/change.3md` and change Contract 34: the lesson bundle is best-effort, read from
  disk, and never undoes archival.
- REQ-change-109: no change source file exceeds 5,000 lines (the touched file has 4,412).
- AGENTS.md: frozen archives are history and are never rewritten.
