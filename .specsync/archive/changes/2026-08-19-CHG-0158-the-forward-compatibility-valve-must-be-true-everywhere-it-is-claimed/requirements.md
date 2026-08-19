---
change: CHG-0158-the-forward-compatibility-valve-must-be-true-everywhere-it-is-claimed
artifact: requirements
---

# Requirements

## REQ-change-079 (modified)

Its cache criterion said "regenerable caches continue to reject a shape they cannot understand"
without saying which files are caches, and the classification behind it was wrong: it counted
`.specsync/agent-artifacts.json`, which is committed and hard-fails rather than rebuilding. The
criterion is narrowed to the file it is actually true of, and the limit that read tolerance does
not extend past a canonical-bytes gate is stated rather than left to be discovered.

## REQ-change-080 (new)

A policy written before a field existed SHALL still load, and each field it does not carry SHALL
take a value that enforces rather than relaxes.

## REQ-agents-005 (new)

A committed agent artifact manifest written by a newer SpecSync of the same major version SHALL
remain usable, while a manifest missing a field this SpecSync requires SHALL still be refused.

See `deltas/change.md` and `deltas/agents.md` for the canonical deltas.

## Deliberately unchanged

Every digest. Both attributes govern deserialization only; no `Serialize` impl moves, so no
preimage moves.

The canonical-bytes gate on the workflow-v2 baseline and the legacy archive baseline. It is
stronger than the attribute on purpose — those two files anchor history and must not drift — and
this change pins that limit with a test instead of removing it.

What a valid record must contain. Every field required before is required now; the manifest and
the policy only stop refusing input they do not need, and the policy's filled-in values are the
enforcing ones.

Read-modify-write preservation. Tolerating an unknown field is not the same as writing it back,
and the rewrite paths still drop what they do not model. Preserving unknowns needs
`#[serde(flatten)]`, which alters the deserialize path of digest preimages, so it is filed
rather than smuggled in beside a comment fix.
