## MODIFIED

### SPEC SECTION Purpose

Links spec files to GitHub issues for traceability. Validates `implements` and `tracks` frontmatter
fields against actual GitHub issues, fetches issue metadata, and creates drift detection issues
when specs fall out of sync. Also defines the maintained composite GitHub Action distribution
contract: immutable exact-version refs and a verified floating major compatibility ref. This
later main wrapper defaults to the qualified published `6.1.1` binary after stable
asset and supported-platform Action checks. Immutable `@v6.1.1` remains on the
qualified candidate SHA with default `6.1.0`; pass explicit `version: '6.1.1'` for
the new stable binary. Floating `@v6` advances only after actual consumer checks
of this separate wrapper commit. The published `@v6.0.0` Action tag still embeds
default `6.0.0-rc.14` and is not
rewritten; `v6` must not be retargeted at that tag. Hosted JavaScript
verification uses one exact supported Bun runtime across site deployment, site CI, and VS Code
extension CI.
