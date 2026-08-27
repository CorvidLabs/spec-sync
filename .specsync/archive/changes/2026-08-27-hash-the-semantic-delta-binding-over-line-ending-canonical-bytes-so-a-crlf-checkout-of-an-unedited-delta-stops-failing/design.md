---
change: hash-the-semantic-delta-binding-over-line-ending-canonical-bytes-so-a-crlf-checkout-of-an-unedited-delta-stops-failing
artifact: design
---

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
