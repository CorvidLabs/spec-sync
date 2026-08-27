---
change: one-delimiter-rule-for-every-frontmatter-reader-at-both-ends-of-the-block
artifact: design
---

# Design

## The shape of the fix

One predicate and one scan, in `src/parser.rs`, used by all three readers that live there.

```rust
fn is_frontmatter_delimiter(line: &str) -> bool {
    line.trim_end_matches([' ', '\t', '\r', '\n']) == "---"
}

fn split_frontmatter(text: &str) -> Option<(&str, &str)> // (yaml block, body)
```

`split_frontmatter` requires the first line to be a delimiter AND to be newline-terminated (a
document that is nothing but `---` opens no block), then scans forward line by line for the first
line that is also a delimiter. It returns borrowed subslices, so nothing allocates.

- `strip_frontmatter` = `split_frontmatter(text).map(|(_, body)| body).unwrap_or(text)`, after the
  existing BOM trim. Its six documented axes are unchanged; only the delimiter rule moved.
- `parse_checked_issue_references` takes the block half instead of its own
  `strip_prefix`/`split_once` chains. It keeps every YAML validation it had.
- `parse_frontmatter` keeps `FRONTMATTER_RE`, with `[ \t\r]*` added after each delimiter.

## Why `parse_frontmatter` keeps its regex

Replacing it with `split_frontmatter` would change TWO behaviours that have nothing to do with this
defect and are relied on across the product: a closing delimiter at EOF with no trailing newline
(`strip` accepts, the regex rejects) and an empty frontmatter block (same). Both are pre-existing
asymmetries between the two readers, both are loud where they differ, and neither was reported. The
narrow change is to the delimiter class only.

The cost is that the rule is now spelled twice — once in `is_frontmatter_delimiter`, once as
`[ \t\r]*` in the regex. That is exactly the sibling-drift risk this repository keeps paying for,
so it is guarded by a test rather than by a comment:
`all_frontmatter_readers_agree_on_what_a_delimiter_is` runs a matrix of opener/closer shapes
through all three readers and fails if any two disagree. Both spellings also carry a comment
pointing at the other.

## Why not option 2 (reject a malformed opener loudly)

`strip_frontmatter` returns `&str` and has no error channel; giving it one means changing every
caller. More importantly, after this change everything still "malformed" is a legitimate Markdown
document: `----` is a thematic break, `---change: x` is text. Erroring on those would fire on valid
documents, which is a worse failure than the residual it would close.

Where an error channel already exists, the readers already fail loudly and keep doing so:
`parse_frontmatter` returns `None`, `parse_checked_issue_references` returns its stable error.

## Why not option 3 (derive the gate from the scaffold)

Checked against `change::artifact_template` and `artifact_content_is_incomplete` rather than
assumed. The pristine scaffold contains an HTML placeholder comment, and the gate short-circuits on
that comment before it ever calls the stripper, so the case a scaffold-equality check would catch
is already caught. (Verified the hard way: the first draft of this artifact quoted the comment
marker verbatim and `change status` reported the artifact itself incomplete.) And the case at
issue — an artifact that is
only frontmatter, opened with `----` — does NOT equal the scaffold, so a scaffold-equality gate
would read it as written. Option 3 does not close the hole it was offered for.

## Behaviour deliberately preserved

- `parse_checked_issue_references` still refuses an EMPTY frontmatter block. The `split_once` it
  replaces could not produce one, so `---` immediately followed by `---` has always been "missing
  or malformed" there, while a block that is a single blank line has always parsed as no
  references. Both verdicts are inherited on purpose and pinned by a labelled CONTROL test, so a
  later tidy-up that collapses them has to argue for it.
- The YAML string handed to `serde_saphyr` is byte-identical to what it received before: the scan
  returns the block including its final line ending, which is trimmed back before the `\n` is
  appended.

## Residual, stated not guessed

A document opened with `----`, `--- x`, `---change: x`, or an indented `---` is returned whole, so
a caller counting prose still sees its YAML as content — including `change`'s completeness gate. It
is characterized by a test that asserts the wrong verdict on purpose and says why closing it would
be worse.
