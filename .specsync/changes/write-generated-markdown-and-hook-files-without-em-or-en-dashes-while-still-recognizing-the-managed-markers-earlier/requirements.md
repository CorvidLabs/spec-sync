---
change: write-generated-markdown-and-hook-files-without-em-or-en-dashes-while-still-recognizing-the-managed-markers-earlier
artifact: requirements
---

# Requirements

The full requirement text is in `deltas/`, one file per module. Approval materializes it.

### REQ-hooks-005 (added)

Hook instruction blocks, the pre-commit hook and Claude Code hook settings are written without
U+2013 or U+2014. The 6.1.1 begin sentinel stays recognized by uninstall. Project-keyed
pre-commit blocks are found by their markers, so a 6.1.1 body is removed as before. Unmarked
legacy pre-commit recognition keeps comparing against the current body; no frozen copy is added.
The new begin sentinel is
``<!-- specsync:hook:begin (managed by `specsync hooks`; do not edit inside) -->``. Detection
markers, the end sentinel and the project-keyed pre-commit markers are unchanged.

### REQ-agents-009 (added)

Generated skill and command files contain neither character. The template version is 6 or
higher, so digest-matching installed copies refresh. The tracked assets in this repository match
the generated output.

### REQ-compact-001 (modified)

The summary row grammar changes from a dash range with dash placeholders to
`<first> to <last>` with `-` placeholders. The 6.1.1 form is still recognized as a prior
summary: it is left byte for byte when nothing is compacted, and is replaced by one current-form
row when more rows fold. A row mixing the two placeholder forms is not a summary. Every other
criterion is restated unchanged.

### REQ-change-116 (added)

The lesson bundle's heading is `# Lesson bundle: <id>`. The bundle's fixed text adds no dash of
its own. All user-supplied record text it copies stays unchanged: the title, acceptance criteria,
affected specs and paths, verification command strings, and the `context.md`, `design.md` and
`testing.md` bodies.

The id is 116, not the next free id on origin/main (113). Two sibling drafts already claim the
next ids: #826 adds REQ-change-113 and #825 adds REQ-change-114 and REQ-change-115. If those
drafts are renumbered or dropped, this id should be revisited before approval rather than leave
a gap by accident.

### REQ-cmd-init-008 and REQ-cmd-migrate-004 (added)

The `.specsync/.gitignore` header lines use a colon. Entries and the no-overwrite rule are
unchanged.

## Human-intent criteria

No existing `hi/` criterion says that generated output avoids particular punctuation. The
nearest are AGENT-5 (instructions installed into each tool in its native form), UPKEEP-1
(trimming a change log) and SETUP-1 (one command sets a project up). None of them is changed or
claimed by this package.

Proposed wording for the human to accept, edit or decline. It is **not** captured, and no id is
spent:

- "Files SpecSync writes into my repository pass my own documentation rules, so its output never
  breaks my docs lint."
- "Upgrading SpecSync never strands a block or summary an older version wrote; the new version
  still recognizes it."
