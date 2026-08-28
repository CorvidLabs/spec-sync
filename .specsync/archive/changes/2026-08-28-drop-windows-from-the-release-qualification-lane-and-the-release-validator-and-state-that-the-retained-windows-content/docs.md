---
change: drop-windows-from-the-release-qualification-lane-and-the-release-validator-and-state-that-the-retained-windows-content
artifact: docs
---

# Docs

`docs/ci-confidence.md`: the confidence-tier table no longer claims Tier B adds Windows, the release
candidate row is Ubuntu + macOS, and the `qualify` row states that Windows is dropped rather than
fixed forward, that the retained `#[cfg(windows)]` code is compiled and run **nowhere**, and that no
candidate has yet qualified — `rc.10` is the first to try.

`CHANGELOG.md`: a new `### Fixed` entry recording the drop, what it costs, why it is a deliberate
trade rather than a discovery that the risk went away, what is retained, and exactly what would
reverse it. The #722 `### Removed` paragraph that argued for keeping the lane is amended in place to
say its argument was correct and is being overruled.

Not changed: `README.md` already says Windows is not a supported target and that
`cargo install specsync` still builds from source, which remains true. The requirement wording in
`specs/cli`, `specs/cmd_migrate` and `specs/commands` already scopes Windows guarantees to platforms
a repository may be *checked out on* rather than to platforms we verify, so it needs no change —
that rebinding was done by #722 and is exactly why nothing now claims a verification that has
stopped happening.
