---
id: write-generated-markdown-and-hook-files-without-em-or-en-dashes-while-still-recognizing-the-managed-markers-earlier
state: draft
type: bug_fix
base_commit: a80748ff44ff42cd7578ac0f7cb8ba7e080ab183
---

# Write generated Markdown and hook files without em or en dashes while still recognizing the managed markers earlier versions wrote

## Intent

Write generated Markdown and hook files without em or en dashes while still recognizing the managed markers earlier versions wrote

## Affected Canonical Specs

- `hooks`
- `agents`
- `compact`
- `change`
- `cmd_init`
- `cmd_migrate`

## Acceptance Criteria

- After `specsync change finalize`, the archived `lesson-bundle.md` opens with `# Lesson bundle: <id>` and contains no U+2013 or U+2014 unless the change's own context.md, design.md or testing.md, which the bundle copies verbatim, contain one.
- `specsync init`, `specsync migrate` (3.x to v4 layout), `specsync hooks install` for every target (CLAUDE.md, .cursorrules, .github/copilot-instructions.md, AGENTS.md, the Git pre-commit hook, .claude/settings.json), `specsync agents install` for every tool, and `specsync compact` summary rows write no U+2013 or U+2014.
- The managed instruction block begins with `<!-- specsync:hook:begin (managed by `specsync hooks`; do not edit inside) -->` and still ends with the unchanged `<!-- specsync:hook:end -->`. `specsync hooks uninstall` removes a block whose begin line is the 6.1.1 form exactly as it removes the new form, preserving content before and after it; `hooks install` and `hooks status` keep treating either form as installed.
- `specsync hooks uninstall --precommit` still removes a legacy unmarked appended block and a legacy whole-file hook carrying the 6.1.1 hook body, and still removes project-keyed managed blocks whose body 6.1.1 wrote.
- A compaction summary row is written as `| <first> to <last> | - | Compacted: N entries <!-- specsync:compact:v1 --> |`. A summary row in the 6.1.1 form (a U+2014 range separator surrounded by spaces, and U+2014 placeholder cells) is still recognized as the prior summary: a no-excess re-run leaves the file byte-for-byte unchanged, and a run that compacts more entries folds its count and range start into one new-form row.
- `AGENT_ARTIFACT_TEMPLATE_VERSION` advances to 6, so `specsync agents install` refreshes create-spec command files whose bytes still match their recorded digest, and the repository's own installed copies and `.specsync/agent-artifacts.json` are regenerated from the built binary.
- A regression test renders every file writer listed above through the built binary in a disposable repository and fails if any written file, other than user-supplied input, contains U+2013 or U+2014. It fails on origin/main a80748ff and passes with the fix.
- Unchanged: terminal, JSON and stdout Markdown output (including `--format markdown` reports, `specsync comment` bodies and issue verification text); the `Spec-Sync Integration` and `Spec-Sync Rules` install-detection markers; the end marker and pre-commit project-keyed markers; frozen archives under `.specsync/archive/`; existing installed blocks and compacted rows, which are not rewritten until the user reinstalls or compaction next folds them.

## No-spec Rationale

Not applicable
