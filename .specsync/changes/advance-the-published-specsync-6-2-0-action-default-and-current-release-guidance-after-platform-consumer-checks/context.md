---
change: advance-the-published-specsync-6-2-0-action-default-and-current-release-guidance-after-platform-consumer-checks
artifact: context
---

# Context

SpecSync 6.2.0 is published on GitHub: release v6.2.0 (promotion run 42043398852, dry run 42043257800) is Latest and not a prerelease, with the ten expected assets. Annotated v6.2.0 and v6.2.0-rc.2 both identify candidate 40dcf33a, qualified on Ubuntu and macos-15 in run 42035620884 (v6.2.0-rc.1 was cancelled at the old 45-minute macOS timeout). crates.io serves 6.2.0. Homebrew is CorvidLabs/homebrew-tap#41, open.

The immutable v6.2.0 tag embeds Action default 6.1.2 (PUBLISHED_ACTION_DEFAULT at release time). Archive-gate examples already pin `@v6.2.0`, and the `@main` allowance ended with 6.1.2, so no archive-gate edit is needed. This change mirrors #836 and its sibling guidance refresh for 6.2.0 in one change, so the release-validator test file is in scope from the start. Floating v6 is currently tag object 189e2c19 at 621a7478 (default 6.1.2) and is not moved by this change.
