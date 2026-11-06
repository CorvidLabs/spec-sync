---
change: hash-a-change-s-clean-delivery-files-one-blob-at-a-time-so-reopen-does-not-overflow-a-single-git-read-on-large-affected
artifact: plan
---

# Plan

## Approach

Stream clean delivery blobs, one at a time, into the acceptance manifest and the acceptance input digest,
instead of loading them all through one capped read, as REQ-change-144 specifies. Failing tests come first.

## Out of scope

- The other deterministic caps: path counts, and the #942/#946 output caps.
- Legacy reconstruction reads, whose failure already becomes the audited `LegacyAcceptanceUnreconstructible`
  cause.
- Making CHG-0026 itself reopenable: about 70 of its inputs drifted, and change-sequence.json cannot be
  owned. Closing it is #833's late close.
- No tag, release or crates.io publish.

## Steps

1. Add the `#[cfg(test)]` cap seam and the failing tests in testing.md. Record that they fail on main
   f7180c34.
2. After scope approval, switch the two builders to object-id discovery and stream each clean blob through
   `git_blob_bytes`.
3. Add the change.3md lesson plane and the CHANGELOG entry.
4. Run `change check --commit`, `change audit --strict`, the verify lane on the pinned rust 1.89, and
   pre-push.
5. As a one-off local check outside CI, reopen CHG-0026 on the throwaway site clone and report the next
   refusal it hits.

## Risks

- **Equality.** If the streamed bytes ever differed from the batched ones, every recorded manifest would
  stop matching. Test 2 pins byte equality across every entry kind.
- **Per-blob spawns.** Outside a fact scope, each `git_blob_bytes` is a separate spawn. Lifecycle commands
  open a scope, so the shared batched reader serves these reads; the verify lane timing will show any
  regression.

## Constraints consulted

- SHIP-6 and its audited reopen.
- REQ-git-utils-010: batched object reads.
- No gate, timeout or assertion is loosened. The one bound that changes is named in design.md: the
  all-at-once total for these two builders becomes a per-blob cap.
