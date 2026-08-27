# Lesson bundle — hash-the-semantic-delta-binding-over-line-ending-canonical-bytes-so-a-crlf-checkout-of-an-unedited-delta-stops-failing

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Hash the semantic delta binding over line-ending-canonical bytes so a CRLF checkout of an unedited delta stops failing the approval gate, and fold nothing else
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change.rs, src/change_tests.rs, specs/change/change.spec.md, specs/change/requirements.md, specs/change/context.md, specs/change/tasks.md, specs/change/testing.md
- **Acceptance**: the approved-delta digest is computed over the delta body with CRLF folded to LF, so a delta a checkout rewrote from LF to CRLF with no other edit still materializes into the canonical spec instead of being refused as changed after approval; nothing beyond line endings is folded, so a real wording change delivered in CRLF is still refused, and a body differing only by a trailing blank line, a leading blank line, trailing spaces, a tab, or a lone carriage return is still refused even though the applier would treat it as equal; and the digest recorded for an LF delta is byte-identical to the digest the raw-bytes binding recorded, so no approval already written since the binding shipped stops verifying

## Evidence

- Verification commit: `d715b24ce1dd5f784ee03139137e7e2723d16bc0`
- Base commit: `d6f266a4fd683246469eb15a8f632061dd5cfbb4`
- Verified by: `cargo test change::`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

# Context

#709 named two remedies. #715 landed the first — the `.gitattributes` `eol=lf` pins for
`.specsync/**/*.md` and `specs/**/*.md` — and said in as many words that the second had not
landed and someone still had to apply it. #711 then shipped the gate that needs it. This change
is that second remedy, filed as #730.

`delta_body_digests` hashed the delta body as raw bytes: `read_bounded_change_text` is a bounded
`fs::read_to_string` that normalizes nothing. But the code that APPLIES a delta treats
line-ending style as explicitly not part of the content, in three independent places:

- `markdown_block_matches` — its doc comment says it compares "ignoring line-ending style and
  surrounding blank lines", and it folds CRLF before comparing;
- `apply_markdown_block` re-emits every body in the target file's own style, trimming trailing
  terminators and re-expanding LF into the file's own ending;
- `parse_delta` reads through `str::lines()`, which discards the carriage return of a CRLF pair,
  so a CRLF delta and an LF delta produce byte-identical canonical specs.

So the module had a definition of when two delta bodies are the same, and the digest binding a
delta to the approval that signed it did not use it. A change approved on Linux and checked out
on Windows with `core.autocrlf=true` produced a different digest with nothing edited, and
`ensure_approved_delta_bodies_unchanged` refused honest work. The remedy that refusal names —
re-approve — re-signs bytes the operator did not choose and diverges again on the next handoff
back. A gate that refuses honest work is worse than an absent gate, because operators learn to
route around it.

## Why this repository could not see it

#715's pins keep spec-sync's own deltas LF on every platform, so the defect is invisible here and
live for every adopter without those pins. We fixed our own instance in #715 and shipped the
class. It also sits inside the guarantee 6.0 explicitly kept when it dropped the Windows binary:
the `### Removed` CHANGELOG entry states the retained case as "a teammate on Windows commits CRLF
files and a colleague on Linux reads them".

## Constraints that shaped the fix

1. **No recorded digest may move.** Measured rather than argued: all 198 archived
   `approvals.json` under `.specsync/` were recomputed under both the raw and the normalizing
   digest. 8 ledgers carry `approved_delta_digests`, 25 module records in total, and the two
   digests are identical for every one of them — 0 move. (Two of the 25 match neither recomputation,
   because they are superseded earlier definition approvals in #711's own ledger, whose delta was
   re-approved twice during that PR; the effective approval matches exactly, and those two differ
   identically under both digests, so this change does not move them either.)
2. **Do not widen beyond line endings.** `markdown_block_matches` also trims surrounding blank
   lines and horizontal whitespace. The digest must NOT copy that half: trailing whitespace and
   blank lines are wording a reviewer signed, and folding them would make the gate accept edits it
   exists to refuse. Only the line-ending axis is provably not content, because it is the only one
   Git rewrites with no author behind it.

## A lone carriage return, decided rather than omitted

Kept as content. Git's `text`, `eol` and `core.autocrlf` conversions only ever move between LF and
CRLF, so no checkout can introduce a classic-Mac terminator; `str::lines()` and
`markdown_block_matches` both keep a bare carriage return as ordinary text, so it reaches the
canonical spec, which makes it wording; and `parser::parse_frontmatter` preserves it deliberately
for the same reason (#715). A body that gained one was edited by a person.

## Sibling sweep

`delta_body_digests` was the only digest in the codebase framing filesystem text. Recorded in
`design.md`; nothing else needs the same change.

## From the change's design.md

# Design

## The change

One helper and one call site.

```rust
fn canonical_delta_body(body: &str) -> Cow<'_, str> {
    if body.contains('\r') {
        Cow::Owned(body.replace("\r\n", "\n"))
    } else {
        Cow::Borrowed(body)
    }
}
```

`delta_body_digests` frames `canonical_delta_body(&body)` instead of `body`. Nothing else moves:
the domain, the module framing, the map shape, the persisted field and every reader of it are
untouched, so this is a change to what one preimage contains and to nothing else.

The `Cow` and its `contains('\r')` guard follow `parser::parse_frontmatter`, which #715 wrote the
same way: an LF document — every delta in a repository carrying #715's pins — borrows and
allocates nothing.

## Why only CRLF, and why that is safe

The applier already defines line-ending style as not content, three times over
(`markdown_block_matches`, `apply_markdown_block`, `parse_delta` via `str::lines()`). Hashing raw
bytes therefore bound the approval to something materialization never consumes. Normalizing
forfeits no security property for exactly that reason: the digest now covers precisely what
rewrites the canonical spec.

## Why nothing else is folded

`markdown_block_matches` normalizes CRLF **and** trims surrounding blank lines, spaces and tabs.
Copying the second half would be a different change with a different sign: the applier asks
"is this edit already applied?", the digest asks "did an approver read these bytes?". Trailing
whitespace and blank lines are wording; Git does not rewrite them; a body that gained one was
edited. Folding them would make the #711 binding accept edits it exists to refuse, which is the
failure mode this release has been bitten by repeatedly — a symptom disappearing because the check
was removed.

A lone carriage return is kept for the same reason and one more: no Git conversion produces one,
so no honest checkout can introduce it, and `str::lines()` carries it into the canonical spec.

## Compatibility

`approved_delta_digests` has only been written since #711, and this repository's deltas are
LF-pinned by #715, so the normalizing digest is byte-identical to every digest already recorded.
Verified by recomputation across all 198 archived `approvals.json` — 25 recorded module digests,
0 move — and pinned in the suite as a literal
(`an_lf_delta_hashes_to_exactly_the_digest_the_unnormalized_binding_recorded`), which passes on
the unfixed binary too and so proves the literal is genuinely the pre-change digest.

## Sibling sweep: nothing else

The report's own worry, checked rather than assumed.

- **Only one digest frames filesystem text.** `grep FramedDigest::new` finds 30 sites, all in
  `change.rs`; `digest.frame(b"body", ...)` occurs exactly once. Of the nine
  `read_bounded_change_text` call sites, eight feed `parse_delta` or an artifact-completeness
  check; only `delta_body_digests` hashed.
- **`approved_scope` / `scope_digest`** (which is `definition_digest` under workflow v2) hashes
  serialized record fields — intent, boundary, answers. No file text at all. Immune by
  construction.
- **`definition_artifact_snapshot`** — the input to the workflow-v1 `definition_digest`,
  `execution_digest`, and by the same evidence path `project_input_digest` and the acceptance
  manifest — does NOT read the working tree for a clean tracked path. `capture_git_candidate`
  takes the payload from the Git **blob** via `cat-file`, and Git stores the blob LF-normalized
  whatever `core.autocrlf` or a `text`/`eol` attribute does to the working tree. The #730 scenario
  (a fresh checkout, files clean) therefore cannot move any of those digests.
- The snapshot falls back to working-tree bytes only through `capture_working_candidate`, taken
  when a path is dirty or untracked. That leaves a narrower dirty-then-clean asymmetry on
  workflow-v1 changes only, and the module already guards the place it matters:
  `append_portable_definition_approval_v501` compares working-tree bytes against the canonical
  blob and refuses with "use a canonical LF release checkout", with
  `portable_projection_rejects_clean_crlf_smudging_before_ledger_mutation` covering it. Not
  widened here; noted so it can be scoped deliberately if it is ever wanted.

## Rejected alternatives

- **Reuse `markdown_block_matches::normalize`.** Rejected above: it trims blank lines and
  horizontal whitespace, which the binding must keep.
- **Compare through `parse_delta` instead of hashing bytes.** Strictly weaker again — it would
  make the digest blind to anything the parser discards, including duplicate-heading shapes the
  module refuses elsewhere, and it cannot be recorded as a stable per-module digest.
- **Rely on `.gitattributes` alone.** That is #715, already landed, and it governs only this
  repository's working trees. An adopter's repository, a tarball, or an archive extracted without
  Git is never covered by it — which is exactly why #709 asked for both remedies.

## From the change's testing.md

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

## Where these lessons go

- `specs/change/context.md`
