# Lesson bundle — keep-3md-files-from-being-classified-as-roff

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep 3md files from being classified as Roff
- **Kind**: BugFix
- **Specs**: cmd_init
- **Paths**: .gitattributes, specs/cmd_init/cmd_init.spec.md, specs/cmd_init/requirements.md, src/commands/init.rs
- **Acceptance**: A new spec-sync project gets *.3md linguist-language=Text, and a project that already classifies *.3md is left alone.

## Evidence

- Verification commit: `3ad7b3ed4114e83d6f3084a5aaf4fc7d16f566c4`
- Base commit: `f60b5ea20f360c8b6db5b536abbc7be88ecffd70`
- Verified by: `specsync check --spec cmd_init`

## From the change's context.md

# Context

GitHub Linguist reads .3* as a man page and a leading dot as a troff macro. The 3md repo forces *.3md to plain text. spec-sync init writes that same rule into every project.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-cmd-init-007 | `adds_3md_linguist_rule_to_missing_gitattributes` and `keeps_an_existing_3md_linguist_rule` pass. |

- `cargo test --bin specsync -- adds_3md_linguist_rule_to_missing_gitattributes` passed.
- `cargo test --bin specsync -- keeps_an_existing_3md_linguist_rule` passed.

## Where these lessons go

- `specs/cmd_init/context.md`
