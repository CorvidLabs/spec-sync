---
change: one-canonical-frontmatter-reader-for-crlf-checkouts
artifact: design
---

# Design

## 1. Normalize inside `parser::parse_frontmatter`

```rust
let normalized: Cow<'_, str> = if content.contains('\r') {
    Cow::Owned(content.replace("\r\n", "\n"))
} else {
    Cow::Borrowed(content)
};
```

The guard matters: every tracked spec in this repository is LF, so the common path allocates
nothing and borrows. The returned `body` is LF-only, which is what all 39 callers already assumed
it was — several index into it, split it on `\n`, and compare section text against LF literals.

A lone `\r` with no `\n` after it is content, not a line ending we produce, and is preserved. That
is pinned by a test so the guard is never "simplified" into stripping every carriage return.

Placement is deliberate: after the BOM trim (a BOM must not hide the opening delimiter either) and
before the regex, so nothing downstream sees mixed endings.

## 2. Promote `change::strip_frontmatter` to `parser::strip_frontmatter`

Four strippers existed. This is the only one correct on all six axes:

| axis | `parser` (regex) | `view` | `change::strip_frontmatter` | `change::strip_yaml_frontmatter` |
|---|---|---|---|---|
| LF | yes | yes | yes | yes |
| CRLF | no (until step 1) | no | **yes** | partly |
| leading BOM | yes | yes | **yes** | no |
| unterminated | returns `None` | keeps document | **keeps document** | keeps document |
| closer at EOF | no | no | **yes** | no |
| body horizontal rule | n/a | yes | **yes** | **NO — deletes content** |

So the move is a promotion, not a rewrite. `view::strip_frontmatter` and
`change::strip_yaml_frontmatter` are deleted rather than left alongside it: the header of
`src/change_tests.rs` records that a fix landing where the report points while a parallel
implementation survives has happened seven times in this release, and #696 was filed to stop the
eighth.

It keeps the borrowed `&str` return and therefore does **not** normalize — a CRLF body comes back
with its carriage returns. That is stated in the doc comment and in the spec, because the
asymmetry with `parse_frontmatter` is exactly the kind of unstated difference this change exists
to remove. Callers needing LF normalize their own input or read through `parse_frontmatter`.

## 3. `.gitattributes`: pin `.specsync/**/*.md`

One line, beside the existing `.specsync/**/*.json text eol=lf` and under the rationale that file
already states. Change artifacts and semantic delta bodies are read as lifecycle evidence; the
JSON pattern never covered the Markdown.

## What this change could NOT do, and why

**#709's second remedy — normalizing `\r\n` inside `delta_body_digests` — is not implemented
here, because `delta_body_digests` does not exist on `main`.** It is introduced by PR #711 (#704,
"a semantic delta must not change after the approval that signed it"), which is still open. The
same is true of the spec wording #709 asks to correct: `specs/change/change.spec.md` invariant 38
saying approval "records a digest over each delta file's **exact bytes**" exists only on that
branch.

Implementing it here would mean either depending on unmerged work or inventing the function this
change has no reason to own. So the honest state is:

- **Landed:** the `.gitattributes` pin, which is the half that prevents the working tree from
  diverging in this repository.
- **Not landed:** the digest-side normalization, which is the half that makes the digest correct
  where the pin is not in force — an adopter's repository, a tarball, an archive extracted
  without Git.

The argument for the normalization is unchanged and should be applied to #711 before or shortly
after it merges: `parse_delta` reads the delta with `content.lines()`, which already discards
`\r`, so a CRLF delta and an LF delta materialize byte-identical specs. Hashing the normalized
form therefore forfeits no security property — it hashes exactly what materialization consumes,
rather than bytes materialization ignores. The "exact bytes" wording describes something the code
does not act on and should be corrected with it.

This limitation is recorded here rather than only in the implementation report, because #709's
closing paragraph is about precisely that failure: a limitation the author knew about that
appears in none of the change's own artifacts.
