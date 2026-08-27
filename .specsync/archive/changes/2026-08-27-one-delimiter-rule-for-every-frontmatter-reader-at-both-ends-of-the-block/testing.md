---
change: one-delimiter-rule-for-every-frontmatter-reader-at-both-ends-of-the-block
artifact: testing
---

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
