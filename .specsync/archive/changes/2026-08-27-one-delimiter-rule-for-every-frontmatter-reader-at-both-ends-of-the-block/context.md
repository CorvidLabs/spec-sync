---
change: one-delimiter-rule-for-every-frontmatter-reader-at-both-ends-of-the-block
artifact: context
---

# Context

## What led here

Issue #716, from an independent review of #715 that swept 1215 artifact shapes through the old and
new strippers and compared verdicts. 293 shapes moved; almost all of them improved. One did not.

`parser::strip_frontmatter` required the opening delimiter to be exactly `---\n` or `---\r\n`, so
`---  \n` with a trailing space was a no-op: the document came back whole and its YAML lines were
body prose. `change::artifact_content_is_incomplete` counts prose lines, so it saw content and
approved an artifact with nothing written in it. #715 closed that hole for well-formed openers and
left it open for malformed ones, and it ships in 6.0.

## What the report got right, and what re-deriving from source added

Measured against a binary built from unfixed `main`, every claim in #716 held:

- `strip_frontmatter("---  \n…")` returns the whole document. Confirmed.
- `strip_frontmatter` with a closer written `---  ` returns `"\nMore prose.\n"` for a body that
  began `"\nReal prose.\n\n---\n\nMore prose.\n"`. **"Real prose." is deleted.** Confirmed, and
  this is the worse half: something extra appearing is loud, prose disappearing is silent.

Three things the report did not say, found by reading the module rather than the report:

1. `parse_frontmatter` has the SAME padded-closer bug and a worse consequence. `FRONTMATTER_RE`'s
   non-greedy `(.*?)\n---\n` walks past a padded closer to the first horizontal rule in the body,
   so `parsed.body` loses the prose above it AND the body lines it swallowed are handed to the
   frontmatter line parser. Measured on unfixed `main`: body `"\nSecond.\n"` and two warnings,
   `Ignoring malformed frontmatter line … \`---\`` and `… \`First.\``.
2. `parse_checked_issue_references` has a third copy of the rule and a fourth bug: its
   `strip_prefix`/`split_once` chains require BOTH delimiters to carry the SAME line ending, so an
   LF-opened, CRLF-closed document is `Err("missing or malformed YAML frontmatter")` when its
   references are right there.
3. #716's option 2 rests on the claim that "`parse_frontmatter` returns `None` today for the same
   inputs, so the two readers already disagree about what those documents are". For a padded
   opener they agree — both say "no frontmatter here". They would only disagree if the stripper
   were loosened and the parser were not, which is the argument for fixing all three together, not
   for adding an error channel.

## Constraints, and what is already ruled out

The strictness is not an accident. `----` is a legal Markdown thematic break; a document that
opens with one is a document. Accepting it as a delimiter makes the scan run forward to the next
rule and return a body cut at it — the failure #697/#699/#705 are about, and one where lost prose
is indistinguishable from prose nobody wrote. So the tolerance had to stop at trailing whitespace
after exactly three dashes, and leading whitespace had to stay refused for the same reason.

Option 3 (make the gate ask whether the artifact still matches its generated scaffold) was checked
against source and does not do what it promises. The pristine-scaffold case is already closed by
`artifact_content_is_incomplete`'s HTML-placeholder-comment short-circuit, which fires before the
stripper runs at all. And a file with a mangled opener no longer equals the scaffold, so a
scaffold-equality gate
would read it as written — the same residual, reached by a different route.
