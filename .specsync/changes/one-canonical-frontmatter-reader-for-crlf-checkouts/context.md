---
change: one-canonical-frontmatter-reader-for-crlf-checkouts
artifact: context
---

# Context

## What led here

`specsync view` hard-fails on a Windows clone. `src/view.rs` reads the spec with
`fs::read_to_string` and hands the raw bytes to `parser::parse_frontmatter`, whose regex is
`(?s)^---\n(.*?)\n---\n(.*)$` — LF only. With `core.autocrlf=true` every spec in the project comes
back as "Cannot parse frontmatter". We ship a Windows binary from `release.yml`; all sixteen CI
jobs are ubuntu-only, so the one platform that breaks is the one never tested.

This is the third correction on issue #696, and the first two were wrong in the same way — a
mechanism asserted from a grep count instead of read call sites:

1. The issue's original table credited `parser.rs` with CRLF support. It has none.
2. The first correction claimed a repository convention of "normalize then parse", counted from
   29 occurrences of `.replace("\r\n", "\n")` against a different denominator. Measured properly:
   **21 of the 39 `parse_frontmatter` call sites outside `parser.rs` normalize, 18 do not.**
   There is no convention; there is a coin flip.

Because there is no convention, "normalize at the boundary" would mean auditing 18 call sites and
creating a permanent, unenforceable obligation whose failure mode is silent. Normalizing inside
the parser fixes all 18 without touching any of them.

## Prior attempts and what is already ruled out

- **#701** taught `change::strip_frontmatter` to accept both encodings directly, keeping its
  borrowed `&str`. That fix is correct and its tests are kept; it just left the repository with a
  fourth dialect instead of one definition. This change promotes that implementation rather than
  rewriting it.
- **Normalizing at every call site** — rejected above.
- **A CRLF-aware regex** — rejected: it multiplies the delimiter grammar across every future
  pattern instead of removing the question, and it would leave the `body` carrying CRLFs that 39
  callers already assume are absent.

## Blast radius, measured rather than assumed

All four strippers were simulated over all 2103 tracked `.md` files in the repository and produced
**zero disagreements**. No tracked file has CRLF or a leading BOM. Unifying them changes output
for zero specs here — it is a pure Windows fix, which is exactly why it survived this long and why
no local test caught it.

## Two of the readers deleted content

Worth naming, because "five implementations that merely differ" understates it:

- `change::strip_yaml_frontmatter` searched the whole document for `\n---\n` **before** trying
  `\r\n---\r\n`, so a CRLF file with one LF horizontal rule in its body lost everything above that
  rule. Its only caller asks "is this artifact written?", so the visible symptom was a completed
  design refused as incomplete.
- The same function only matched a closing delimiter followed by a newline, so frontmatter closed
  at end of file was not stripped at all and its own `---` and `change:` lines read as prose. An
  artifact with no content passed the completeness gate.

## Deliberately out of scope

Steps 4 and 6 of the #696 migration order:

- `commands/lifecycle.rs:26` uses an unanchored `find("---\n")` and can therefore edit a `status:`
  line in the BODY rather than in frontmatter. A real, orthogonal bug; a different module; not
  widened into this change.
- A source-grep test forbidding new `strip_prefix("---` outside `parser.rs`.

And the part of #709 this change cannot reach: see `design.md`.
