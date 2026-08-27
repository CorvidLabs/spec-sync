---
change: one-delimiter-rule-for-every-frontmatter-reader-at-both-ends-of-the-block
artifact: research
---

# Research

## Method

A checkout of unfixed `main` (`d6f266a4`) was exported to a separate directory and built. Every
claim below is a measurement against that binary, not a reading of the code.

## Every frontmatter/delimiter reader in the repository

| Site | Kind | Delimiter rule before | Verdict |
|------|------|-----------------------|---------|
| `parser::strip_frontmatter` | reader | `strip_prefix("---\n"\|"---\r\n")`, closer `trim_end_matches(['\r','\n']) == "---"` | FIXED — the reported site |
| `parser::parse_frontmatter` (`FRONTMATTER_RE`) | reader | `^---\n(.*?)\n---\n(.*)$` | FIXED — same padded-closer bug, worse consequence |
| `parser::parse_checked_issue_references` | reader | `strip_prefix` + `split_once`, per line ending | FIXED — same bug plus a mixed-line-ending bug |
| `commands/lifecycle.rs::update_status_in_content` | WRITER | unanchored `content.find("---\n")`, then `rest.find("\n---")` | LEFT — see below |
| `registry.rs::extract_module_name` | line scanner | `line == "---"`, stop on `starts_with("---")` | LEFT — see below |
| `merge.rs:358` (`in_frontmatter`) | line counter | `l.trim() == "---"` | LEFT — already whitespace-tolerant, and it only counts |
| `agents.rs`, `comment.rs`, `change.rs::artifact_template`, `generator.rs` | writers | emit `---\n` | LEFT — they produce the canonical shape |

### Measured behaviour on unfixed `main`

```
strip_frontmatter("---  \nchange: CHG-1\nartifact: design\n---\n")
  => "---  \nchange: CHG-1\nartifact: design\n---\n"      (whole document)
strip_frontmatter("----\nchange: CHG-1\nartifact: design\n---\n")
  => "----\nchange: CHG-1\nartifact: design\n---\n"       (whole document)
strip_frontmatter("---\nspec: a.spec.md\n---  \n\nReal prose.\n\n---\n\nMore prose.\n")
  => "\nMore prose.\n"                                    (Real prose. DELETED)
parse_frontmatter("---\nmodule: auth\nversion: 1\n---  \n\n# Auth\n\nFirst.\n\n---\n\nSecond.\n")
  => module=Some("auth") body="\nSecond.\n"
     warnings=["Ignoring malformed frontmatter line (expected `key: value`): `---`",
               "Ignoring malformed frontmatter line (expected `key: value`): `First.`"]
parse_frontmatter("---  \nmodule: auth\nversion: 1\n---\n\n# Auth\n")            => None
parse_checked_issue_references("---  \nimplements: [1]\n---\n\nBody.\n")
  => Err("missing or malformed YAML frontmatter")
parse_checked_issue_references("---\nimplements: [1]\n---  \n\nBody prose.\n\n---\n\nMore.\n")
  => Err("invalid YAML frontmatter")                       (body prose reached the YAML parser)
parse_checked_issue_references("---\nimplements: [7]\r\n---\r\n\r\nBody.\r\n")
  => Err("missing or malformed YAML frontmatter")           (mixed line endings)
```

### Sites deliberately left, and why

- **`commands/lifecycle.rs::update_status_in_content`** is a WRITER, not a reader, and its defect
  is orthogonal to the delimiter shape: `content.find("---\n")` is unanchored, so on a document
  with no frontmatter but a horizontal rule in its body it can rewrite a `status:` line in the
  prose. #715 named it out of scope; routing it through the canonical scan means returning byte
  ranges rather than slices, which is a different change. Confirmed it is not made worse here: for
  a padded opener it now simply fails closed with "could not find status line", and for a padded
  CLOSER it already produced the right answer. #715 said this was "named in `tasks.md`"; it is not
  in `specs/cmd_lifecycle/tasks.md` — that referred to a change artifact since archived — so it is
  recorded in `specs/parser/tasks.md` here instead.
- **`registry.rs::extract_module_name`** is a line scanner that reads a `module:` value; it cannot
  delete body content. With a padded opener it stops at the first line and returns `None`, so the
  module is simply not auto-registered — fail-closed and cosmetic.
- **`merge.rs:358`** already uses `l.trim() == "---"`, which is at least as tolerant as the new
  rule, and it only counts delimiters to decide whether it is still inside frontmatter.

## Why the delimiter tolerance stops where it does

Jekyll, gray-matter and front-matter all accept `\A---\s*\r?\n`: exactly three dashes plus trailing
horizontal whitespace. None of them accept four. A document whose first line is `----` is a
document that opens with a thematic break, and treating that as an opener makes the scan run to the
next rule and return a body cut at it. That is not a hypothetical: it is the shape of #697's
`split("---")` truncation and of the two strippers #715 deleted.

Leading whitespace is refused for the same reason and because no frontmatter implementation accepts
it: an indented `---` inside a document is a list continuation or a code block, not a delimiter.
