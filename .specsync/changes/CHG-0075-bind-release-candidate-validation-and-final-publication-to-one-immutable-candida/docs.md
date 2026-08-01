---
change: CHG-0075-bind-release-candidate-validation-and-final-publication-to-one-immutable-candida
artifact: docs
---

# Docs

- `docs/ci-confidence.md` now presents the approved Ubuntu-development and immutable cross-platform
  RC policy as the active workflow contract rather than a future roadmap item.
- Canonical implementation guidance lives in `specs/github/github.spec.md`, `requirements.md`,
  `context.md`, and `testing.md` alongside the workflow contract.
- The hosted workflow exposes one discoverable promotion input, `rc_tag`; qualification starts by
  creating a new annotated `vX.Y.Z-rc.N` marker, and any candidate change requires a new marker.
- Final release ordering is explicit and enforced: qualify three platforms, dispatch promotion,
  create the final tag, revalidate tags/evidence/artifacts, then upload.
- Repository setup uses a dedicated CorvidLabs release GitHub App with repository `Contents: write`
  permission. Store its numeric id as the repository variable `SPECSYNC_RELEASE_APP_ID`; store its
  private key only as `SPECSYNC_RELEASE_APP_PRIVATE_KEY` in the protected `release` environment.
  Restrict that environment's deployment branch policy to the protected default branch (`main`);
  promotion also rejects a workflow dispatch whose workflow ref is not that branch.
- Configure `SpecSync immutable RC tags` for `refs/tags/v*.*.*-rc.*` with update/deletion rules and
  no bypass. Configure `SpecSync final tag creation` for final semver refs (excluding RC refs) with
  a creation rule and only the configured release App integration bypass. Configure `SpecSync
  immutable final tags` over those final refs with update/deletion rules and no bypass.
