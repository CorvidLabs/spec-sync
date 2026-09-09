---
change: complete-specsync-6-promotion-and-public-release-contracts
artifact: context
---

# Context

The independent Claude Fable 5.1 review ran through CorvidLabs Rune on public PR761 snapshot456e1a62 and returned conditional GO at about70 percent subjective confidence. It found promotion-count drift: REQUIRED_PLATFORMS is Ubuntu/macOS, but both promotion and publication shell guards require three records. Both existing guards were executed locally with two receipt filenames and exited1. RC16 has real successful receipts for both required platforms at ffba9a32. No stable release has been published.

This change also corrects remaining authoritative public surfaces: the CLI and canonical contract overclaim local review authentication, README/CLI examples lag the slug workflow, SECURITY omits6.x, and6.0 release notes remain under Unreleased. The separate approved PR761 guidance correction is in progress and must be merged first; rebase this work onto its merged tree before implementation verification. Do not overwrite that work.

## Implementation progress

Both workflow guards now require the existing Ubuntu/macOS evidence set. Fifty-two release-validator tests pass; restoring the old count in either guard fails its corresponding regression. Built CLI help and existing same-actor/pass-block and exact-succession tests pass. Site lint, 23 site tests, and the redirect build pass. Changed Markdown was rendered separately under /private/tmp/specsync-promotion-rendered; the 6.0 release heading is unique and undated, and release-version validation passes. Full local verify is running; canonical materialization, final baseline integration, and release gates are still pending.

The approval CLI initially rejected malformed delta headings; their syntax was corrected without changing the approved wording, and the explicit human scope approval is now successfully recorded as 0xLeif. No self-granted approval or review was recorded.

Separate finding: all three published executable demos fail with retired IDs. Their repair has its own prepared scope under /private/tmp/specsync-6-examples and is awaiting approval. PR761 correction at65de8d9d has all hosted checks green but still awaits the requested human implementation review before final archival and merge.
