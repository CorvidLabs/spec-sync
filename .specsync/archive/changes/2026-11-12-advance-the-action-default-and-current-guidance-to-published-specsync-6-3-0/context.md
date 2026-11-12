---
change: advance-the-action-default-and-current-guidance-to-published-specsync-6-3-0
artifact: context
---

# Context

SpecSync 6.3.0 is published.

- **The release:** GitHub release `v6.3.0` is Latest with twelve assets (promotion run 43168578964). It was promoted from `v6.3.0-rc.1` at `b87bbd88`, which qualified on Ubuntu and macos-15 (run 43162493255).
- **Authority:** Leif's go relayed by hc t370 (2026-11-12). t370 asked for this post-publication defaults change, as #867 was for 6.2.0.
- **Timing:** `cargo publish` is on hold for Leif's separate crates.io go, and the guidance here says crates.io serves 6.3.0. So this PR merges only after that publish.
- **Elsewhere:** the Homebrew bump is CorvidLabs/homebrew-tap#42. Moving floating `v6` is a separate decision for Leif.
