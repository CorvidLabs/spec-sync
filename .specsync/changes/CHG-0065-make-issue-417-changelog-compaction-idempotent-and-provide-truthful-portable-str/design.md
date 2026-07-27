---
change: CHG-0065-make-issue-417-changelog-compaction-idempotent-and-provide-truthful-portable-str
artifact: design
---

# Design

## Compaction ownership

A row is tool-owned only when it carries `<!-- specsync:compact:v1 -->`, has a non-empty range,
valid placeholders, and a grammatical fixed-width count. Unmarked lookalikes remain ordinary
history; multiple marked rows are rejected. Backslash parity and code-span delimiters govern pipe
splitting, and only one contiguous width-valid changelog table is processed.

## Byte stability

The changelog section is reconstructed without changing content outside that section. A no-op
second run returns the original bytes. Inclusive source-line reconstruction preserves each original
LF/CRLF terminator, including mixed-ending files. Checked `u64` arithmetic rejects summary overflow.

## Output contract

The root dispatcher forwards the resolved global `OutputFormat` to both maintenance commands.
Command wrappers own presentation:

- text preserves the existing terminal-oriented output;
- JSON emits exactly one ANSI-free document and separates `would_change` from `applied`;
- Markdown and GitHub emit a heading, optional dry-run notice, table, and summary.

Structured renderers normalize separators only on Windows. Unix literal backslashes keep their
identity. Markdown/GitHub sanitizes controls/bidi marks, escapes table pipes, and selects a code-span
delimiter longer than every embedded backtick run.

## Safety

Dry-run delegates collect plans without writing. Apply mode inspects every input and stages
same-directory temporary replacements before publication. Preflight failure writes nothing; late
publication/rollback failure is explicit incomplete/partial evidence and exits 1 after rendering.
Empty results remain successful no-ops.
