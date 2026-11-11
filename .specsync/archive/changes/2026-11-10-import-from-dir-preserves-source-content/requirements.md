---
change: import-from-dir-preserves-source-content
artifact: requirements
---

# Requirements

- A markdown file with frontmatter imported via `--from-dir` keeps its `module`, `files`, `version`, and `status` values.
- The source's Purpose body and any existing Public API / Invariants / Behavioral Examples / Error Cases / Dependencies / Change Log sections survive import; only missing sections are scaffolded.
- The imported spec passes `specsync check --strict` with no frontmatter failure.
- The Change Log entry names the actual source (local directory import) and never stamps Confluence for a non-Confluence source.
- A source file with no determinable `files` list fails the item loudly and the batch exits non-zero.
