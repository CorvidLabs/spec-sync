# Plan

Two edits, both narrow.

## Tolerance for evidence, strictness for caches

Remove `#[serde(deny_unknown_fields)]` from the seventeen persisted-evidence structs. Keep it in
`hash_cache.rs` and `agents.rs`, which discard and rebuild on an unrecognised shape — that is
correct and costs nothing, because a cache can be recomputed and evidence cannot.

The distinction is the design: **tolerance is for what cannot be recreated.**

## An unknown version names its remedy

Three sites rejected an unrecognised `workflow_version`. Each now says the record was written by
a newer SpecSync and that the reader should be upgraded, and none describes it as invalid.

## What this does not buy, stated rather than glossed

`ApprovedScopeV1`, `CorrectionRecord` and `ScopedReviewRecord` are digest preimages —
`scope_digest`, the correction digests, and `finalization.review_digest`. Adding a field to one
of those still changes its serialized bytes and therefore its digest. Tolerance lets an older
reader *parse* such a file instead of erroring; it does not make field addition digest-safe for
those three.

Removing the attribute changes no digest at all, because `deny_unknown_fields` governs
deserialization and serialization is untouched.
