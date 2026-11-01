---
change: write-generated-markdown-and-hook-files-without-em-or-en-dashes-while-still-recognizing-the-managed-markers-earlier
artifact: tasks
---

# Tasks

- [x] Add the discriminator and control tests in `testing.md`, and confirm their labels against an unfixed a80748ff build.
- [x] Write `# Lesson bundle: <id>` in `write_lesson_bundle`.
- [x] Use a colon in the `init` and `migrate` `.specsync/.gitignore` headers.
- [x] Repunctuate `CREATE_SPEC_STEPS_MD` and set `AGENT_ARTIFACT_TEMPLATE_VERSION` to 6.
- [x] Repunctuate the four hook snippets, and write the new begin sentinel while reading both forms in `remove_section_from_file`.
- [x] Repunctuate `PRE_COMMIT_HOOK`. No reader changes: project-keyed blocks are found by their markers.
- [x] Write compaction summaries as `<first> to <last>` with `-` placeholders, and read both forms in `summary_metadata`.
- [x] Tie the range separator to its placeholder form, and refuse a range start that contains ` to ` (PR #829 review).
- [x] Add `tests/integration/generated_output_has_no_dashes.rs` and register it in `tests/integration.rs`.
- [x] Regenerate this repository's create-spec commands and the command entries of `.specsync/agent-artifacts.json` with the built binary.
- [ ] Regenerate the four tracked `*/skills/spec-sync/SKILL.md` files (`.claude`, `.codex`, `.cursor`, `.gemini`) and every entry of `.specsync/agent-artifacts.json` with the built binary, so every tracked agent asset and manifest entry matches a fresh install and REQ-agents-009 criterion 3 holds. Add `tracked_agent_artifacts_and_manifest_match_a_fresh_install` (see `testing.md`) first and confirm it fails at bb7b4b4d before regenerating. 0xLeif decided to add the four paths to `affected_paths` and regenerate (PR #829); this task runs after that amendment is re-approved.
- [x] Update the compact requirements list, the hooks, compact and agents lesson planes, and each touched spec's version and Change Log.
- [x] Run the regression lanes in `testing.md`, re-run the issue's repro with the fixed build, and record the observed results.

Context for the open task above: the tracked `SKILL.md` files for all four tools no longer
match their recorded digests or the generated `SKILL_BODY` (drift from #808, before this
change), so `agents install` in this repository refuses each tool. Only the command entries
of `.specsync/agent-artifacts.json` and the two create-spec commands were regenerated at
bb7b4b4d. The amended `affected_paths` now lists the four `SKILL.md` files beside
`.specsync/agent-artifacts.json`, so the regeneration stays inside the approved scope.

## Human gates

Scope approval was recorded by 0xLeif (PR #829) with the draft's recommended answers to the
open questions below.

Implementation review (`specsync change review <id> --reviewer <human>`) by a human follows a
current `change check`. Finalization, merge and any release are separate human decisions.

Decisions proposed to the human, not made by this draft:

- Whether to capture the human-intent wording proposed in `requirements.md`, and under which ids.
- The compaction range separator (` to ` is proposed; `..` is the alternative) and placeholder
  (`-` is proposed).
- Whether the hook begin sentinel should be recognized by an explicit two-form list (proposed)
  or by the prefix `<!-- specsync:hook:begin`.
- Whether `hooks install` should, in a later change, refresh an already-present managed block so
  existing installs lose their dashes without an uninstall.
- Whether the guard test should also drive `scaffold`, `generate`, `import` and `init-registry`.
  A byte grep of their literals found no dash today.
- Whether to fix, in a separate change, the existing gap found while drafting:
  `hooks uninstall --precommit` has not recognized the unmarked hooks that v5.x wrote since
  #774 changed `PRE_COMMIT_HOOK`. Those installs are now reported as an ambiguous legacy block
  or left alone. This draft does not change that behavior.
- Requirement id order with the sibling drafts. This package takes REQ-change-116, after #826
  (REQ-change-113) and #825 (REQ-change-114 and REQ-change-115). If the human prefers a
  different order, the ids are renumbered before any of the three is approved.
- Whether stdout Markdown that users commonly redirect into files, such as `specsync comment`
  bodies and `--format markdown` reports, should follow the same rule in a later change.

## After verification

Run `fledge lanes run pre-push` before every push, and `fledge trust verify` before calling the
work complete. Hosted CI and Trust results for the exact head are recorded as evidence, not
assumed.
