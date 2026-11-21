---
change: advance-the-action-default-and-current-guidance-to-published-specsync-6-4-1
artifact: context
---

# Context

- SpecSync 6.4.1 is published (2026-11-21): GitHub release `v6.4.1`, Latest, twelve assets, promoted from `v6.4.1-rc.1` at `3f0985e8` (release.yml runs 44843175474 qualify, 44851306915 dry run, 44854914914 promote); crates.io serves 6.4.1, and the Homebrew tap PR is CorvidLabs/homebrew-tap#44.
- As #1136 did for 6.4.0: the main Action wrapper's default and the current guidance move to the published release. The immutable `@v6.4.1` tag stays at `3f0985e8`, where `action.yml` defaults to `6.4.0`.
- Leif's answer, 2026-11-20 about 22:41Z, relayed by hc t4c: "Go (Recommended)" to releasing 6.4.1 with the same steps as 6.4.0, which named "then the Action default and @v6"; t4c confirmed it covers this change.
