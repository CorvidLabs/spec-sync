---
change: CHG-0060-prepare-the-specsync-5-2-0-feature-release-bump-accurate-release-metadata-and-c
artifact: context
---

# Context

Main carries five shipped features since v5.1.1: the native `migrate 5.0` change-ledger backfill
(#404), batch `change correct-owner` (#403), inert 5.0.1 registry stub tolerance (#405),
squash-merged accepted-evidence archival (#400), and adoption-era legacy archive repair (#402).
The 22-repo Trust rollout that surfaced these frictions still runs 5.1.1, so the fixes only reach
users through a published release. All five change records are accepted and archived (#406), and
`check --strict` is green, so the release preparation starts from a clean ledger.

This change performs the in-repository release preparation only: metadata, changelog, Action
default, maintained consumer pins, and documentation, plus the full verification and trust lanes.
Tagging, publication, and post-publication smoke tests are defined as bounded follow-up steps but
are not executed here.
