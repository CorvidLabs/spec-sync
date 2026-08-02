---
change: CHG-0075-bind-release-candidate-validation-and-final-publication-to-one-immutable-candida
artifact: context
---

# Context

CHG-0074 documents the approved confidence model but intentionally leaves protected workflows
unchanged. Today, `.github/workflows/ci.yml` runs Ubuntu, macOS, and Windows for ordinary pull
requests, while `.github/workflows/release.yml` starts only after a final `v*` tag already exists.
That spends cross-platform runner time during development and can discover a release failure after
the final tag has been published.

Trigger: the user approved Ubuntu as the ordinary integration authority and required cross-platform
validation before final release publication.

Root cause: development CI and release qualification share one always-on platform matrix, and no
immutable RC identity connects platform evidence to later final-tag creation.

Invariant: one immutable RC marker names one candidate commit; every required platform result and
the final release tag must resolve to that exact unchanged commit.

Audit finding: post-merge archive binding is emitted by `post-merge-archive.yml` from a
`pull_request` event, but the current release validator requires `pull_request_target`. That
event mismatch rejects valid release provenance and must be corrected with a focused regression
inside the existing `release.yml` scope.

This is a protected-workflow change. It must use the repository's separately pinned required-workflow
process and must not be folded into CHG-0074.

Implementation checkpoint: ordinary PR integration now has one Ubuntu authority. The Release
workflow resolves an annotated RC marker, rejects conflicting workflow history for a deleted or
recreated marker name, validates merged archive provenance, and runs the same
`release-candidate` Fledge lane on Ubuntu, macOS, and Windows. Its aggregate check is useful for
visibility, but promotion also downloads and revalidates the three source records; publication
then re-hashes six packaged archives against SHA-bound provenance manifests. This keeps check-run,
artifact, tag, and checkout identity independently testable.

Independent review finding: a workflow-level ordering promise is insufficient while humans can
create or move matching final tags, mutable Action refs execute with write permissions, executable
checksums come from the same mutable release, or publication trusts an earlier tag resolution.
The repaired boundary requires three active tag policies: human creation but no update/deletion for
RC markers, dedicated-release-GitHub-App-only creation for final tags, and final-tag
update/deletion protection with no bypass for any actor. The
private key is scoped to the protected `release` environment's promotion job, which mints a
short-lived installation token restricted to the repository; checkout credentials are disabled.
The boundary also
requires full-SHA Action pins, embedded Fledge/jq digests, overwrite-safe reruns, and fresh
RC/final-tag, actual-HEAD, original-evidence, and artifact revalidation immediately before upload.
Stable-tag pushes are not a Release workflow trigger.

Live sandbox repair: `CorvidLabs/spec-sync-sandbox` now exposes three distinct active rulesets and
a `release` environment restricted to `main` with administrator bypass disabled. The production
validator accepts those exact API
responses. The sandbox creation policy still uses the existing Codex integration only as a bounded
stand-in; the dedicated CorvidLabs release App, its environment-only private key, production
environment, and production rulesets remain an explicit activation task rather than being claimed
as completed evidence.

Final security-review invariant: publication treats only an explicit authenticated 404 as proof
that a release is absent; transient/API failures retry and then fail closed. Candidate Python that
prepares qualification evidence runs isolated without persisted checkout credentials or the
check-writing token, which is exposed only to the final inline publication step.
