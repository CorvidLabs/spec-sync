---
change: one-canonical-frontmatter-reader-for-crlf-checkouts
artifact: research
---

# Research

## The counts, measured rather than inferred

Issue #696 was corrected twice, both times for asserting a mechanism from a grep count. The
numbers that survive scrutiny:

```
parse_frontmatter call sites outside src/parser.rs : 39
  of which normalize before calling               : 21
  of which do not                                 : 18
occurrences of .replace("\r\n", "\n") in src/      : 29   (different denominator; not a convention)
tracked .md files                                  : 2103
  of which have CRLF or a leading BOM              : 0
```

The last row is why this is invisible locally and why all four strippers agree on every real file
in the repository. It is also why the fix must be justified by reading the platform, not by
observing a local difference: there is none to observe.

## Nine readers, not five

The issue's opening table listed five and was wrong in four of five rows. Counting the readers
that actually decide where frontmatter ends:

- `parser.rs` — `FRONTMATTER_RE`, LF-only (fixed here)
- `parser.rs` — the `serde-saphyr` checked-issue delimiter extraction, already CRLF-tolerant
- `view.rs:135` — LF-only, no closer at EOF (deleted here)
- `change.rs:6200` — correct on all six axes (promoted here)
- `change.rs:8043` `strip_yaml_frontmatter` — content deleter (deleted here)
- `commands/lifecycle.rs:26` — unanchored `find("---\n")`, can edit the BODY (out of scope, step 4)
- `registry.rs:444` — line-wise, no BOM handling
- plus the two call sites inside the lessons feature that #701 already collapsed onto one helper

## Why the failure is silent in both directions

Unstripped frontmatter renders as noise; over-stripped frontmatter deletes body content. Neither
raises an error, and the deleting one is the more dangerous: in a lesson bundle or a change
artifact, truncated material is indistinguishable from material nobody ever wrote. That
asymmetry — a "formatting" bug that is really a content-integrity bug — is why this was worth
unifying rather than patching the reader that happened to be reported.

## #709: why hashing the normalized form forfeits nothing

`parse_delta` reads a delta with `content.lines()`. `str::lines` splits on `\n` and discards a
trailing `\r`, so a CRLF delta and its LF twin produce byte-identical materialized specs. A digest
over raw bytes therefore distinguishes two inputs the product cannot distinguish, and refuses
honest work with no content change behind it. An independent review computed both digests for a
real delta on the #704 branch: `LF b30dfb39…` versus `CRLF 0b9bd896…`.

It fails closed with an actionable message, so this is availability rather than security. But a
gate that refuses honest work teaches people to work around the gate, and the cross-OS handoff
that triggers it — approve on Windows, accept on Linux; or a branch switch in a clone with
`core.autocrlf=true` — is a normal thing for a team to do.

This change lands the `.gitattributes` half. The digest half cannot land here: see `design.md`.
