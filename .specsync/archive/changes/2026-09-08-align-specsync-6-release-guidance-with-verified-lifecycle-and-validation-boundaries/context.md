---
change: align-specsync-6-release-guidance-with-verified-lifecycle-and-validation-boundaries
artifact: context
---

# Context

RC16 source passes the full local verification lane and hosted PR checks. This audit found public documentation still describing the pre-6.0 lifecycle and insufficiently explicit boundaries. REQ-change-055 and the old sequence guidance are already removed from current canonical guidance; current downgrade and review-evidence tests pass. The remaining change is public documentation only. Model review and macOS qualification remain separate release evidence, not approval substitutes.

## Implementation evidence

The seven scoped pages now distinguish structural checks from behavioral proof, content-bound approvals from authenticated identity, current slug workflows from legacy recovery, and finalization from GitHub merge. No executable or canonical contract files changed.

Validation: Astro check reported zero errors/warnings/hints; all 23 site tests passed; the site built 43 redirect pages. Because the standalone site redirects to the CorvidLabs hub, the changed Markdown was also rendered locally with the existing marked dependency and inspected separately. This does not claim the hub has deployed these edits. Strict SpecSync passed all 62 specs with 100 percent source coverage. Full local verification and Trust are recorded separately after completion.
