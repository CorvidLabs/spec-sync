---
change: pin-the-trust-gate-to-1-2-1
artifact: context
---

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
