---
change: advance-the-action-default-and-current-guidance-to-published-specsync-6-4-2
artifact: context
---

# Context

- SpecSync 6.4.2 is published (2026-11-24): GitHub release `v6.4.2`, Latest, twelve assets, promoted from `v6.4.2-rc.2` at `a275ab44` (release.yml runs 45472594205 qualify, 45489088237 dry run, 45489261983 promote); crates.io serves 6.4.2, and the Homebrew tap carries it (CorvidLabs/homebrew-tap#45, merged).
- As #1142 did for 6.4.1: the main Action wrapper's default and the current guidance move to the published release. The immutable `@v6.4.2` tag stays at `a275ab44`, where `action.yml` defaults to `6.4.1`.
- The 6.4.2 preparation (#1156) already moved the YAML examples to `@v6.4.2` with `version: '6.4.2'` and left three sentences reading "after publication"; they read as published here.
- Leif asked for the cargo and Homebrew publication steps in the release-lead chat on 2026-11-24 and is asked there to approve this scope; his words are in the approval record.
