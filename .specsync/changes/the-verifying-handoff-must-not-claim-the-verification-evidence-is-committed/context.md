---
change: the-verifying-handoff-must-not-claim-the-verification-evidence-is-committed
artifact: context
---

# Context

Found while running the coverage record on this branch through the lifecycle. After
`specsync change check <id>` (without `--commit`) the record was Verifying with
`verification.json` untracked, and the new `Handoff:` line read:

    Handoff: safe — verification is committed and current; a fresh session resumes at the independent review

The verdict is right: `ship-status` reports the product tip as done and the review stage as
current from uncommitted evidence, and a fresh session reads `verification.json` from disk, so
nothing this session knows is lost by clearing. But `classify_handoff` reaches that arm on
`verification_current` (content currency, `recorded_verification_is_current`) and
`scoped_edits_uncommitted != Some(true)` (the implementation paths are clean). `HandoffSignals`
has no "verification committed" signal, so the reason asserts something the classifier never
checked, and does so in the most common state — every `change check` run without `--commit`.

The fix is wording only: say what was checked — the implementation is committed and its
verification is current. The Approved/Implementing arm already words it that way.

Ruled out: making the arm `conditional` with "run `change check --commit`". The lifecycle does
not require the evidence to be committed before review, and telling agents to commit between
review and finalize is exactly what `ship-status` warns against.
