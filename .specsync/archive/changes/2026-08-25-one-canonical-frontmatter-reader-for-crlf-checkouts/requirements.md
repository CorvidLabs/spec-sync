---
change: one-canonical-frontmatter-reader-for-crlf-checkouts
artifact: requirements
---

# Requirements

- `parse_frontmatter` accepts CRLF input and returns an LF-only `body`, so no caller has to
  normalize first. The normalization is guarded on the presence of a carriage return, so an LF
  document allocates nothing.
- A lone carriage return with no line feed after it is content and is preserved.
- `specsync view` renders a CRLF-authored spec, and its `requirements.md` companion, exactly as it
  renders their LF twins. A Windows checkout is never a parse failure.
- Exactly one frontmatter stripper exists in the repository, `parser::strip_frontmatter`. It is
  correct on LF, CRLF, a leading BOM, unterminated frontmatter, a closing delimiter at end of
  file, and a horizontal rule in the body — all six together, not each in isolation.
- `view::strip_frontmatter` and `change::strip_yaml_frontmatter` are deleted, not deprecated and
  not left alongside the canonical one.
- Artifact completeness is decided by the canonical stripper: a written CRLF artifact is never
  refused as incomplete, and an artifact that is only frontmatter closed at end of file is never
  accepted as written.
- Markdown under `.specsync/` is pinned to `eol=lf` in `.gitattributes`, beside the existing JSON
  pin and under the rationale that file already states.
- The change makes no behavioural difference to any of the 2103 tracked Markdown files in this
  repository: none has CRLF or a leading BOM. It is a Windows and adopter fix, and is expected to
  read as a no-op locally.
