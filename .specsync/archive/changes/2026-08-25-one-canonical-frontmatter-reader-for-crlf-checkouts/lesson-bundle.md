# Lesson bundle — one-canonical-frontmatter-reader-for-crlf-checkouts

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: One canonical frontmatter reader for CRLF checkouts
- **Kind**: BugFix
- **Specs**: parser, view, change
- **Paths**: src/parser.rs, src/view.rs, src/change.rs, src/change_tests.rs, specs/parser/parser.spec.md, specs/parser/context.md, specs/view/view.spec.md, specs/view/context.md, specs/change/change.spec.md, specs/change/context.md, .gitattributes
- **Acceptance**: specsync view renders a CRLF spec instead of failing with 'Cannot parse frontmatter'; parse_frontmatter accepts CRLF input and always returns an LF-only body while an LF document still allocates nothing; a single parser::strip_frontmatter is correct on LF, CRLF, a leading BOM, unterminated frontmatter, a closer at EOF and a body horizontal rule, and view::strip_frontmatter plus change::strip_yaml_frontmatter are deleted rather than left in parallel; a CRLF change artifact with a written body is no longer refused as incomplete and a frontmatter-only artifact closed at EOF is; .specsync/**/*.md is pinned to eol=lf beside the existing JSON pin; cargo fmt, cargo clippy -D warnings and the full test suite pass.

## Evidence

- Verification commit: `35027aafdcc71721fed277bcd6cc8535ebe47d28`
- Base commit: `e82542d19ce8d79926b144a0e38d4d620b120715`
- Verified by: `cargo test change::`, `ruby --version`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

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

## From the change's design.md

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

## From the change's testing.md

# Testing

Every fixture the repository already had was LF, which is why a shipped Windows failure survived
2103 tracked Markdown files and sixteen CI jobs. So each test below is recorded with what it
discriminates against — and each was **run against the pre-change implementation and observed to
fail** before being accepted. Tests that are invariants or controls rather than discriminators are
labelled as such, in the repository's convention.

## Discriminators — verified red before the fix

| Test | Discriminates against | Observed failure before the change |
|---|---|---|
| `parser::tests::test_parse_frontmatter_crlf_document` | the LF-only `FRONTMATTER_RE` | `parse_frontmatter` returned `None`; `expect` panicked |
| `parser::tests::test_parse_frontmatter_crlf_with_bom_and_body_horizontal_rule` | same, with the exact fixture #696 names (CRLF + BOM + body rule) | returned `None`; `expect` panicked |
| `view::tests::test_view_spec_renders_a_crlf_spec` | the shipped defect end to end | `view_spec` returned `Err("Cannot parse frontmatter")` |
| `view::tests::test_view_spec_strips_frontmatter_from_a_crlf_requirements_companion` | `view::strip_frontmatter` being LF-only | the companion's raw YAML block was rendered under `## Requirements` |
| `parser::tests::test_strip_frontmatter_all_six_axes` | every stripper that was not correct on all six | verified red against the old LF-only `view` implementation |
| `change::tests::a_crlf_artifact_with_an_lf_body_rule_is_complete_when_its_prose_is_written` | `strip_yaml_frontmatter` deleting everything above a body rule | a fully written design was reported **incomplete** |
| `change::tests::an_artifact_that_is_only_frontmatter_closed_at_eof_is_incomplete` | `strip_yaml_frontmatter` not stripping a closer at EOF | an artifact with **no content at all** was reported complete |

The two `change` tests were re-run with `strip_yaml_frontmatter` temporarily restored and its
caller repointed: both failed, in opposite directions. The `view` companion test and the six-axis
test were re-run with the old LF-only stripper spliced in: both failed. The two parser tests were
re-run with the normalization guard forced to `false`: both failed.

## Controls and invariants — labelled, not counted as evidence

| Test | Label |
|---|---|
| `parser::tests::test_parse_frontmatter_lf_body_is_unchanged_by_the_crlf_guard` | CONTROL. Asserts the LF body byte-for-byte, so the guard cannot start rewriting the common path. Passes before and after. |
| `parser::tests::test_parse_frontmatter_preserves_a_lone_carriage_return_in_the_body` | INVARIANT. A lone `\r` is content. Passes before and after; it exists so a later "simplification" to `retain(\|c\| c != '\r')` is caught. |
| `parser::tests::test_strip_frontmatter_keeps_a_document_that_has_none` | CONTROL. A document with no frontmatter comes back untouched, rules and all. |
| `view::tests::test_view_spec_renders_an_lf_spec` | CONTROL for the two CRLF view tests. |
| `change::tests::artifact_completeness_verdicts_are_unchanged_for_lf_artifacts` | CONTROL. This is a swap of implementations, not of policy: the LF verdicts must not move. |
| `change::tests::strip_frontmatter_removes_crlf_frontmatter_and_keeps_later_rules` (existing, from #701) | Kept. Still discriminates — it fails against an LF-only stripper — and now guards the promoted implementation. |

## Not covered

- **No test exercises a real Windows checkout.** CI is ubuntu-only across all sixteen jobs, and
  this change does not add a Windows job. Every CRLF case above is a synthesized fixture. That is
  the same gap that let the defect ship, narrowed but not closed: a fixture proves the reader
  handles CRLF bytes, not that `core.autocrlf=true` produces the bytes we think it does.
- **The `.gitattributes` pin is untested by construction.** It changes what Git writes into a
  working tree, which no unit test in this repository observes.
- **`delta_body_digests` normalization is absent, not untested** — see `design.md`.

## Commands

`cargo build --release`, `cargo fmt`, `cargo clippy -- -D warnings` (bare, not `--all-targets`,
which has pre-existing failures in test code), `cargo test`, then `specsync change check`.
Clippy is not among the project's configured verification commands, so `change check` does not
run it and it was run separately.

## Where these lessons go

- `specs/parser/context.md`
- `specs/view/context.md`
- `specs/change/context.md`
