---
change: CHG-0060-prepare-the-specsync-5-2-0-feature-release-bump-accurate-release-metadata-and-c
artifact: design
---

# Design

Version is 5.2.0 (minor: five backward-compatible features, no breaking changes). Every
in-repository version surface moves together in one commit so the deterministic release
validators stay green: `Cargo.toml`/`Cargo.lock`, `action.yml` default, `ci.yml` and `trust.yml`
consumer pins, README installation examples, and the site quickstart/integration docs.
`CHANGELOG.md` gains a 5.2.0 section generated from accepted change records and reviewed against
the merged PRs. The `github` canonical spec gains REQ-github-004 documenting the 5.2.0 Action
promotion contract (immutable exact ref, floating `v5` promotion only after exact-version
artifacts pass supported-platform verification).

Publication boundaries stay fail-closed: the tag, GitHub Release, crates.io, Homebrew, and the
floating `v5` Action ref are out of scope for this change and proceed only after the accepted
release commit lands on main, in monotonic order with per-step verification and the documented
rollback rule (delete the unpublished tag/ref; never rewrite published artifacts).
