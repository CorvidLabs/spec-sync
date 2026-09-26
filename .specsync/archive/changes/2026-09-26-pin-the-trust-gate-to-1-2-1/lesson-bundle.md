# Lesson bundle — pin-the-trust-gate-to-1-2-1

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Pin the Trust gate to 1.2.1
- **Kind**: Operations
- **Paths**: .github/workflows/trust.yml
- **Acceptance**: The Trust gate step in .github/workflows/trust.yml resolves CorvidLabs/trust@dd52a7a90ffbc1d2b1030e37007c18274f3d96bc (v1.2.1, the peeled commit of the annotated v1.2.1 tag) instead of fcc889f54d8b4892a81af463c5a0250e2be66fc5 (v1.2.0). The runner-local file:// SpecSync mirror still works unchanged: Trust downloads specsync-linux-x86_64.tar.gz and its .sha256 from specsync-download-base-url, revalidates the checksum, and gates on the pull request's own freshly built binary. specsync-version stays 6.0.0. Nested Augur and Attest install their prebuilt Linux binaries (augur-linux-x86_64, attest-linux-x86_64) instead of building Swift from source, and the hosted trust check passes.

## Evidence

- Verification commit: `c656982771a869edc29ff8f8e95d2d7aff95d6ca`
- Base commit: `cddc39e478dcc1f111940a3cfb02134bba9804cc`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

## What led here

This repository's own Trust gate was pinned to
`CorvidLabs/trust@fcc889f54d8b4892a81af463c5a0250e2be66fc5` (`v1.2.0`). Trust
1.2.0 nests Augur and Attest at tips that, when run inside Trust, looked for
their release assets in the calling action's repository rather than in
`CorvidLabs/augur` and `CorvidLabs/attest`. `gh release download` then missed
`augur-linux-x86_64` and `attest-linux-x86_64`, and both tools fell back to
compiling Swift from source on every hosted `trust` run.

Trust `v1.2.1` (`dd52a7a90ffbc1d2b1030e37007c18274f3d96bc`, the peeled commit of
the annotated tag object `25f18128`) re-pins the nested actions to
`CorvidLabs/augur@108a2ff4` and `CorvidLabs/attest@2182cae4`, which set
`RELEASE_REPO` to their own repository and download the prebuilt Linux
binaries. That is the only runtime change between `v1.2.0` and `v1.2.1`
(`action.yml`: two `uses:` lines).

PR #792 carried the one-line pin but failed `trust` and `Lifecycle gate`
because `.github/workflows/trust.yml` is a meaningful path and no active change
covered it. This change record covers it.

## Constraints a session picking this up needs to know

The Trust step here is not a normal consumer pin. The workflow builds the pull
request's own SpecSync binary, packages it as a checksum-protected runner-local
mirror under `${RUNNER_TEMP}`, and passes
`specsync-download-base-url: file://${{ runner.temp }}/specsync-trust-mirror`,
so Trust gates on the pull request's binary rather than on a published release.
`scripts/trust_cli.py`, which resolves and downloads that mirror, is the same
blob (`a82229c5`) at `v1.2.0` and `v1.2.1`, so the mirror path is unchanged.

## Ruled out

Trust `v1.2.2`. Its only runtime change re-pins Augur to a tip that reports a
failed prebuilt download on Darwin/arm64 accurately; it does not affect the
Linux runner this workflow uses. Moving to it is a separate change.

Changing `specsync-version`, the mirror packaging steps, or the
`specsync change audit --strict` preflight. None is stale.

## From the change's testing.md

# Testing

There is no unit test for a CI pin. Verification is a source comparison of the
two Trust revisions plus the hosted run itself.

## Verified before the ref was changed

- `gh api repos/CorvidLabs/trust/commits/v1.2.1` resolves to
  `dd52a7a90ffbc1d2b1030e37007c18274f3d96bc`; the `v1.2.1` ref points at tag
  object `25f18128`, which peels to that commit.
- `gh api repos/CorvidLabs/trust/compare/fcc889f5...dd52a7a9`: the only
  `action.yml` change is the nested Augur pin (`25ef9339` to `108a2ff4`) and
  the nested Attest pin (`e8a2d928` to `2182cae4`).
- `scripts/trust_cli.py` has the same blob SHA (`a82229c5`) at both
  revisions, so the `file://` SpecSync mirror resolution, checksum
  revalidation, and `specsync-download-base-url` input are unchanged.
- `action.yml` at `CorvidLabs/augur@108a2ff4` and
  `CorvidLabs/attest@2182cae4` sets `RELEASE_REPO` to its own repository and
  selects `augur-linux-x86_64` / `attest-linux-x86_64` on x86_64 Linux,
  falling back to `swift build` only when that download fails.

## Verified locally on the committed tree

- `.github/workflows/trust.yml` still parses as YAML.
- The product diff against `main` is one changed line in one file.
- `fledge lanes run pre-push` passes.

## Verified on the hosted run

The pull request's own `trust` check is the real test. A passing run proves
that Trust 1.2.1 resolved, installed SpecSync 6.0.0 from the `file://` mirror,
accepted the locally built archive's checksum, and ran the contract, risk, and
provenance stages. Its log should show `Installed augur (augur-linux-x86_64,
1.0.0)` and `Installed attest (attest-linux-x86_64, 1.0.0)` with no Swift
source build.

Both `trust` and `Lifecycle gate` run `specsync change audit --strict` before
the gate itself, which is why this change record exists:
`.github/workflows/trust.yml` is a meaningful path and the audit fails closed
when no active change covers it.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
