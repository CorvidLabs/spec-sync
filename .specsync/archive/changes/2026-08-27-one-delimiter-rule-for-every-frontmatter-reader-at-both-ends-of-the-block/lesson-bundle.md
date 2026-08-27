# Lesson bundle — one-delimiter-rule-for-every-frontmatter-reader-at-both-ends-of-the-block

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: One delimiter rule for every frontmatter reader, at both ends of the block
- **Kind**: BugFix
- **Specs**: parser, change
- **Paths**: src/parser.rs, src/change_tests.rs, specs/parser/parser.spec.md, specs/parser/context.md, specs/parser/tasks.md, specs/parser/testing.md, specs/change/change.spec.md, specs/change/context.md, specs/change/testing.md
- **Acceptance**: A frontmatter delimiter line is three dashes plus trailing whitespace at BOTH ends of the block, in either line encoding, with the two ends free to disagree, and strip_frontmatter, parse_frontmatter and parse_checked_issue_references all apply that one rule; an artifact that is only frontmatter opened with a padded delimiter is refused by the completeness gate instead of approved; prose above the first horizontal rule in a body survives a padded closing delimiter in every reader; a line that is not exactly three dashes — four dashes, an indented three, or three followed by text — is still not a delimiter in any reader; and the two behaviours PR #715 changed without stating them, a BOM-prefixed empty artifact being refused and parse_frontmatter returning an LF body for a CRLF-only body, are pinned by test and written into the specs

## Evidence

- Verification commit: `d6f266a4fd683246469eb15a8f632061dd5cfbb4`
- Base commit: `d6f266a4fd683246469eb15a8f632061dd5cfbb4`
- Verified by: `cargo test change::`, `ruby --version`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

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

## From the change's design.md

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

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-parser-003` | `all_frontmatter_readers_agree_on_what_a_delimiter_is` proves the three readers return the same verdict across a matrix of opener/closer shapes. `test_strip_frontmatter_accepts_a_delimiter_padded_with_trailing_whitespace`, `test_parse_frontmatter_opens_on_a_delimiter_padded_with_trailing_whitespace` and `test_parse_frontmatter_closes_on_a_delimiter_padded_with_trailing_whitespace` prove the padded-delimiter behaviour at each end, including that no body prose reaches the frontmatter line parser. `test_parse_checked_issue_references_uses_the_one_delimiter_rule` proves the padded opener, the padded closer above a body rule, and the mixed-line-ending pair. `test_strip_frontmatter_refuses_a_delimiter_that_is_not_three_dashes` proves `----`, `--- x`, `---change: x` and an indented `---` are still not delimiters. `test_parse_checked_issue_references_keeps_its_empty_block_verdicts` proves the empty and blank-line block verdicts are unchanged. `test_parse_frontmatter_body_is_lf_even_when_only_the_body_is_crlf` proves the LF body for a CRLF-only body. In `change`, `an_artifact_that_is_only_frontmatter_with_a_padded_opening_delimiter_is_incomplete`, `a_padded_closing_delimiter_does_not_delete_the_prose_above_a_body_horizontal_rule`, `a_bom_prefixed_artifact_with_no_written_body_is_incomplete` and `a_four_dash_opener_still_hides_an_empty_artifact_from_the_gate` prove the approval-gate consequences and the stated residual. |

## Discrimination protocol

A checkout of unfixed `main` (`d6f266a4`) was exported to a separate directory and built. The new
test functions — and only the test functions, not the fix — were spliced into that checkout and
run. The fix was never reverted in place.

Seven assertions FAIL there. Five pass there and are labelled CONTROL or CHARACTERIZATION; they are
not counted as evidence.

## DISCRIMINATOR — recorded failure against unfixed `main`

| Test | Failure on the control binary |
|------|-------------------------------|
| `test_strip_frontmatter_accepts_a_delimiter_padded_with_trailing_whitespace` | `assertion left == right failed: a padded OPENER must open the block, or its YAML counts as prose` / `left: "---  \nchange: CHG-1\nartifact: design\n---\n"` / `right: ""` |
| `test_parse_frontmatter_closes_on_a_delimiter_padded_with_trailing_whitespace` | `body: "\nSecond.\n"` — "First." deleted, and `warnings` holds two `Ignoring malformed frontmatter line` entries naming body prose |
| `test_parse_frontmatter_opens_on_a_delimiter_padded_with_trailing_whitespace` | `a padded opener still opens frontmatter` (the `expect` on a `None`) |
| `test_parse_checked_issue_references_uses_the_one_delimiter_rule` | `called Result::unwrap() on an Err value: "missing or malformed YAML frontmatter"` |
| `all_frontmatter_readers_agree_on_what_a_delimiter_is` | `strip_frontmatter disagrees for opener "---  " / closer "---"` / `left: false` / `right: true` |
| `an_artifact_that_is_only_frontmatter_with_a_padded_opening_delimiter_is_incomplete` | `assertion failed: artifact_content_is_incomplete("---  \nchange: CHG-1\nartifact: design\n---\n")` — the empty artifact is approved |
| `a_padded_closing_delimiter_does_not_delete_the_prose_above_a_body_horizontal_rule` | `written prose above a body horizontal rule must survive a padded closing delimiter` — the written design is refused as incomplete |

Control run: `5 passed; 7 failed`.

## CONTROL — passes on the control binary, and must

| Test | What breaks if someone "fixes" it |
|------|-----------------------------------|
| `test_strip_frontmatter_refuses_a_delimiter_that_is_not_three_dashes` | Generalising the tolerance to "starts with three dashes" makes `----`, `--- x`, `---change: x` and an indented `---` into openers. Every document that opens with a Markdown thematic break then has its body cut at the next rule, and lost prose reads like prose nobody wrote. |
| `test_parse_checked_issue_references_keeps_its_empty_block_verdicts` | Routing the checked reader through the shared scan must not change WHICH documents it accepts beyond the delimiter shape. An empty block was refused and a blank-line block accepted; collapsing them silently changes a security-sensitive reader. |

## CHARACTERIZATION — passes on the control binary, which is the point

| Test | What it records |
|------|-----------------|
| `a_bom_prefixed_artifact_with_no_written_body_is_incomplete` | The third bug #715 fixed without claiming: this module's old stripper had no BOM trim, so a BOM-prefixed empty or TODO-only artifact passed the gate. Correct fix, no test, unstated. |
| `test_parse_frontmatter_body_is_lf_even_when_only_the_body_is_crlf` | #715 made `parse_frontmatter` normalize whenever the document contains a `\r`, so an LF-frontmatter/CRLF-body document now returns an LF body where it returned CRLF. Every consumer was traced as read-only analysis; this is the record of what they were promised. |
| `a_four_dash_opener_still_hides_an_empty_artifact_from_the_gate` | The KNOWN RESIDUAL, asserting a wrong verdict on purpose. If it ever fails, check the reason and delete it — do not restore the behaviour. |

## Suites

- `cargo test` — 2407 unit, 407 integration, all green.
- `cargo fmt --check` clean; `cargo clippy -- -D warnings` (the CI invocation) clean.
- `specsync check --strict --require-coverage 100` — 62/62, 100% file and LOC coverage.
- `cargo clippy --all-targets -- -D warnings` reports 20 findings, every one of them pre-existing
  on `main` at `d6f266a4` and none in a file this change touches. Verified by running the same
  command in the control checkout.

## Where these lessons go

- `specs/parser/context.md`
- `specs/change/context.md`
