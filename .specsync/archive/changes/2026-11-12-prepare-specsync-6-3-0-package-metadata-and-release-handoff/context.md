---
change: prepare-specsync-6-3-0-package-metadata-and-release-handoff
artifact: context
---

# Context

Leif decided to release (coordinator chat, 2026-11-12, relayed by hc t370): "yes chough 0.6.0 ships with spec-sync 6.3.0". The coordinator assigned this preparation as task 2 and marked it "Do not merge until Leif says go". No tag, release, publish or workflow dispatch happens without his explicit go.

- **6.3.0, not 6.2.1:** main has added commands since `v6.2.0` (doctor, retire, repair-relays, relayed decisions, `--timeout`), and a new download (aarch64 musl).
- **Starting points:** claude-code-t37j's 6.2.1 draft (`_audits/spec-sync-6.2.1-draft-2026-11-11.md`), reworked to 6.3.0, with everything merged since included. The 6.2.0 precedent is #859, release prep, and #867, the post-release Action default.
- **#1067**, the relay gate (#1064), changes the relayed-decision CHANGELOG entry and adds the upgrade note. It must merge to main, and main must be merged into this branch, before the RC tag. That merge is the one update this preparation still waits for. The note: "once a repository records one relayed decision, every machine that runs change commands there needs SpecSync 6.3.0 or later; check-only uses keep working".
