---
change: CHG-0009-make-accepted-evidence-squash-safe-and-harden-the-5-0-release-path
artifact: design
---

# Design

Keep commit ancestry as the primary proof. When squash merging replaces the recorded commit, accept the existing
closing evidence only if the current scoped-input digest still matches and the complete accepted workspace is already
tracked unchanged at the remote default ref containing HEAD. This makes the remote integration—not a mutable local
claim—the bounded fallback. Factor remote-default discovery so coverage and evidence validation agree.

Archive the six already-merged accepted workspaces after validating them from a clean checkout of the squash commit.
Narrow release tag matching, validate tag/package/main consistency, and pin the Action's default binary major.
Use an anchored Cargo `include` allowlist so local ignored files and repository-only assets cannot enter the crate.

Harden every lifecycle digest with an explicit version/domain prefix and length-prefixed fields. Hash stable file
kinds and Git-relevant executable mode while preserving arbitrary binary bytes. This closes structural ambiguity
without weakening stale-evidence detection through generic line-ending normalization. Temporary Git topology
fixtures set their own checkout policy so Windows runner defaults cannot mutate the fixture after evidence is recorded.
