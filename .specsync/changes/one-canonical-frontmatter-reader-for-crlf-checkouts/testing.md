---
change: one-canonical-frontmatter-reader-for-crlf-checkouts
artifact: testing
---

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
