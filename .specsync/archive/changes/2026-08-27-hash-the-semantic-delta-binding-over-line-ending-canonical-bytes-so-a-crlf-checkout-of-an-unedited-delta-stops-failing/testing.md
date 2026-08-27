---
change: hash-the-semantic-delta-binding-over-line-ending-canonical-bytes-so-a-crlf-checkout-of-an-unedited-delta-stops-failing
artifact: testing
---

# Testing

Every new assertion was judged against a binary built from a **separate checkout of unfixed
`main`** at `d6f266a4`, not by reverting the fix in place. The tests were spliced into that
checkout verbatim (minus two helper-unit assertions that name a function which does not exist
there) and run with `cargo test --bin specsync`.

## DISCRIMINATOR — 1

`a_delta_a_checkout_rewrote_to_crlf_still_reaches_the_canonical_spec`

Approve a change with an LF delta, re-encode that delta to CRLF and nothing else, require
materialization to succeed and the approved wording to reach `specs/auth/auth.spec.md`. The
fixture derives the CRLF body from the approved one and asserts the round trip in both directions,
so it cannot drift into "some other text that happens to contain CRLF". It also asserts the
materialized spec carries no carriage return, which is the independent evidence that line-ending
style really is not content here.

Verbatim, against the unfixed control binary:

```
thread 'change::tests::a_delta_a_checkout_rewrote_to_crlf_still_reaches_the_canonical_spec'
panicked at src/change_tests.rs:13737:10:
a delta whose line endings a checkout rewrote was not edited by anybody: "semantic delta for
`auth` changed after approval; the approved wording is what rewrites the canonical spec, so
re-run `specsync change approve add-passkeys` to approve the current delta bodies (or restore
them)"
```

That is the #711 gate refusing honest work, which is the defect.

## CONTROLS — 4, all honestly labelled, all passing on the unfixed binary

Passing on the control is the point of every one of them: they say what the fix may NOT do.

- `a_reworded_delta_is_refused_even_when_it_arrives_with_rewritten_line_endings` — **the important
  one.** "Normalize everything" would satisfy the discriminator. This delivers a real wording
  change (`BACKDOOR: this text was never reviewed or approved by anyone.`) in CRLF and requires
  the refusal to stand, the module to be named, the canonical spec to stay clean, and
  `canonical_applied` to stay false. If the normalization is ever widened to fold the body as a
  whole, this is what fails.
- `a_delta_edited_only_in_whitespace_the_applier_would_ignore_is_still_refused` — the boundary
  marker against `markdown_block_matches`, which also trims surrounding blank lines and horizontal
  whitespace. Four bodies the applier would call equal to the approved one — a trailing blank
  line, a leading blank line, trailing spaces on a content line, a tab indenting one — each must
  still be refused. This is the assertion that fails if someone reuses the applier's `normalize`.
- `a_lone_carriage_return_is_delta_content_and_is_not_folded_away` — CHARACTERIZATION of a
  decision taken deliberately. A bare carriage return is wording: no Git conversion produces one,
  and `str::lines()` carries it into the canonical spec. Also pins the helper directly — an LF
  body takes the borrowed `Cow` path, and `"a\r\nb\rc\n"` folds to `"a\nb\rc\n"`.
- `an_lf_delta_hashes_to_exactly_the_digest_the_unnormalized_binding_recorded` — COMPATIBILITY.
  The pre-change digest is pinned as a literal
  (`66d9882e0429aff9d0dc043c78e84b526e021e9678693967e423fdd06f00734a`). It passing on the unfixed
  binary is what proves the literal is genuinely the raw-bytes digest, so the assertion is not
  circular.

## Recomputation across the archive, not reasoning about it

All 198 archived `approvals.json` under `.specsync/` were recomputed under both digests using the
framed SHA-256 preimage (`domain` / `module` / `body`). 8 ledgers carry `approved_delta_digests`;
25 module records in total; the raw and the normalized digest are identical for **every one**, so
**0 move**. No archived delta file contains a CRLF pair or a lone carriage return.

Two of the 25 match neither recomputation. Both are earlier, superseded definition approvals in
#711's own ledger — that PR's delta was re-approved twice after rebase edits, which #711 reported
at the time — and the effective (last) definition approval matches the archived delta exactly.
They differ identically under both digests, so this change moves nothing there either.

## Suite

`cargo test`: 2400 unit + 407 integration, 0 failures. `cargo fmt --check` clean. CI-equivalent
`cargo clippy -- -D warnings` clean.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-089 | Five tests in `src/change_tests.rs`, each judged against a binary built from a SEPARATE CHECKOUT of unfixed `main` at `d6f266a4` rather than by reverting the fix in place. DISCRIMINATOR `a_delta_a_checkout_rewrote_to_crlf_still_reaches_the_canonical_spec` — approve an LF delta, re-encode it to CRLF and nothing else (the fixture asserts the round trip both ways), require materialization to succeed; unfixed it panics with `a delta whose line endings a checkout rewrote was not edited by anybody: "semantic delta for \`auth\` changed after approval; the approved wording is what rewrites the canonical spec, so re-run \`specsync change approve add-passkeys\` to approve the current delta bodies (or restore them)"`. CONTROL `a_reworded_delta_is_refused_even_when_it_arrives_with_rewritten_line_endings` — a real swap delivered in CRLF is still refused, BACKDOOR never reaches `specs/auth/auth.spec.md`, `canonical_applied` stays false; this is the assertion "normalize everything" would break while still passing the discriminator, and it passes on the unfixed binary too. CONTROL `a_delta_edited_only_in_whitespace_the_applier_would_ignore_is_still_refused` — four bodies `markdown_block_matches` would call equal (trailing blank line, leading blank line, trailing spaces, a tab) must all still be refused, pinning that the digest folds ONLY the line-ending axis and not the applier's blank-line and horizontal-whitespace trimming. CHARACTERIZATION `a_lone_carriage_return_is_delta_content_and_is_not_folded_away` — a bare CR is wording, kept deliberately because no Git conversion produces one and `lines()` carries it into the canonical spec; also pins the helper (LF borrows, `"a\r\nb\rc\n"` folds to `"a\nb\rc\n"`). COMPATIBILITY `an_lf_delta_hashes_to_exactly_the_digest_the_unnormalized_binding_recorded` — the pre-change digest `66d9882e0429aff9d0dc043c78e84b526e021e9678693967e423fdd06f00734a` pinned as a literal; it passes on the unfixed binary, which is what proves the literal is genuinely the raw-bytes digest. Blast radius measured rather than argued: all 198 archived `approvals.json` under `.specsync/` recomputed under both preimages — 8 ledgers carry digests, 25 module records, 0 move, and no archived delta contains a CRLF pair or a lone CR. `cargo fmt --check` clean, CI-equivalent `cargo clippy -- -D warnings` clean, 2400 unit and 407 integration tests pass |
