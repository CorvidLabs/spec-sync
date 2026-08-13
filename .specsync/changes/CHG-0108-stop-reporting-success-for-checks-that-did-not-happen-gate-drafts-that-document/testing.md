---
change: CHG-0108-stop-reporting-success-for-checks-that-did-not-happen-gate-drafts-that-document
artifact: testing
---

# Testing

## Strategy

Three of these fixes *remove* output — a warning, a class of warning, an error. The risk is
never that the fix fails to fire but that it silences something that should still speak. So
each carries a paired negative test, and for the draft gate the negatives are the existing
suite.

## The draft gate — the pinned contracts are the test

The three integration tests that pin "a draft passes `--strict`" —
`draft_planned_mapping_passes_strict_and_is_absent_from_coverage`,
`draft_mapping_transitions_on_activation_and_file_creation`, and
`draft_dot_segment_mapping_transitions_to_covered_file` — **pass unedited**.

That is the load-bearing result. An earlier, broader rule (warn on any draft with present
source) failed exactly those three, which is how the rule was found to be wrong. Their
fixture has a Public API with headers and no rows, so the narrow rule lands in a row no
existing test occupies.

Verified by hand across all three shapes:

| fixture | `--strict` | draft warning |
|---|---|---|
| draft, source present, documents `nonexistent_function` | **exit 1** | 1 |
| draft, `src/future.rs` not created yet | exit 0 | 0 |
| draft, source present, empty Public API | exit 1 *(pre-existing empty-section warning)* | **0** |

The third is worth its own line: it exits 1 for an unrelated, pre-existing reason — an
empty `## Public API` counts as an unfinished section — and the draft warning correctly
stays silent. Reading only the exit code would have made this look like a false positive.

## Tests added

### Quoting — `src/parser.rs`

| Test | Asserts |
|---|---|
| `quoted_block_list_items_and_scalars_are_unquoted` | `files:`, `depends_on:`, `module:` and `status:` all resolve inside the quotes, single and double, mixed with unquoted entries |
| `quoted_entry_keeps_a_trailing_comment_out_of_the_path` | `- "src/auth.ts" # the main file` yields the path only |
| `a_hash_inside_a_quoted_path_is_not_a_comment` | `- "src/a#b.ts"` keeps the `#` |
| `an_unterminated_quote_is_an_error_not_a_literal_path` | the negative case — a loud error, and the value is not retained |
| `flow_style_lists_still_unquote_their_own_items` | the pre-existing path is untouched |

### Cache — `src/hash_cache.rs`

| Test | Asserts |
|---|---|
| `a_cold_cache_selects_for_revalidation_without_claiming_drift` | both halves in one test: cold cache still returns `is_changed`, `baseline_known` is false and `reportable` is false; then warms the cache, edits `requirements.md`, and asserts the warning **comes back** |
| `quoted_frontmatter_files_are_cached_under_the_real_path` | the second mini-parser agrees with the first |

The first is deliberately one test rather than two — silence and signal are the same
property observed twice, and splitting them invites deleting the half that fails.

## Results

- `cargo fmt --all -- --check` — clean
- `cargo clippy -- -D warnings` — exit 0
- `cargo test` — **2210 unit, 331 integration, 0 failures**

Verified against real repositories: a fresh clone of `fledge` reports 0 phantom drift
warnings where it previously reported 33, and one appended line in
`specs/ai/requirements.md` on a warm cache produces exactly one correctly-named warning.

## Not covered here

`cargo clippy --all-targets` fails on pre-existing lint debt in test code unrelated to this
change (`tests/integration/change.rs` collapsible-if, and two `manual_contains` sites in
`src/change.rs` tests). CI runs `cargo clippy -- -D warnings` without `--all-targets`, so
this is long-standing and invisible to the pipeline. Noted rather than fixed, to keep this
change's scope honest.

Issue #546 (symlink abort) is filed, verified, and deliberately not fixed here.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-commands-006 | `cargo test`; the three pinned draft tests pass unedited, proving spec-first authoring and the empty-API stub still pass `--strict`. Verified by hand: a draft with present source documenting `nonexistent_function` exits 1 with exactly one draft warning; the same spec with the API row removed produces zero draft warnings |
| REQ-types-005 | `cargo test`; `ValidationResult` carries `had_present_source` and `documents_contract`, both set before the draft skip, which is what lets the three-way rule distinguish the cases at all |
| REQ-validator-011 | `cargo test`; `had_present_source` is set only in the `SourceSnapshot::Present` and ambient `is_file()` branches. Missing, planned, directory, unreadable and escape mappings reach earlier branches and cannot set it — verified by the directory-mapping and planned-mapping tests continuing to pass |
| REQ-hash-cache-002 | `cargo test`; `a_cold_cache_selects_for_revalidation_without_claiming_drift` asserts selection survives, reporting stops, and a real edit against a warm baseline reports again. `quoted_frontmatter_files_are_cached_under_the_real_path` pins the second parser. Confirmed on a fresh `fledge` clone: 33 phantom warnings before, 0 after, 1 for a genuine edit |
| REQ-cmd-check-005 | `cargo test`; both reporting sites moved to `reportable`, and the JSON `stale` array, the warning count and the review hint all populate inside that branch, so every format follows. Spec selection is untouched — the cold-cache run still validates all 33 specs |
| REQ-parser-002 | `cargo test`; five tests covering block items, scalars, trailing comments, a `#` inside quotes, unterminated quotes, and the untouched flow-style path |
| REQ-change-064 | `cargo test`; the remediation names at most twelve paths and summarizes the remainder with a covering-prefix suggestion |
