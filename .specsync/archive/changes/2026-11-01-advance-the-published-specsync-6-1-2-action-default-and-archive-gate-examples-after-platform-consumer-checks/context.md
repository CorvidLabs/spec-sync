---
change: advance-the-published-specsync-6-1-2-action-default-and-archive-gate-examples-after-platform-consumer-checks
artifact: context
---

# Context

SpecSync 6.1.2 is published on GitHub: release v6.1.2 (promotion run 41348603304) is Latest and not a prerelease, with the ten expected assets. Annotated v6.1.2 (tag object d16ef670) and v6.1.2-rc.2 (27e3c761) both identify candidate d08d2746, qualified on Ubuntu and macos-15 in run 41345066214. The macOS aarch64 archive matched its sidecar and prints `specsync 6.1.2`. crates.io publication is run by the owner; it is a precondition for approval of this change. Homebrew is homebrew-tap#40.

The immutable v6.1.2 tag embeds Action default 6.1.1 (PUBLISHED_ACTION_DEFAULT at release time). Archive-gate examples (`require-archived`, `archive-gate-only`) pin `@main` because `@v6.1.1` ignores those inputs; 6.1.2 is the first release that has them. This change mirrors #817 for 6.1.2. Floating v6 is currently tag object 189e2c19 at 621a7478 (default 6.1.1) and is not moved by this change.
