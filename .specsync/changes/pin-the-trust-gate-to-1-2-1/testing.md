---
change: pin-the-trust-gate-to-1-2-1
artifact: testing
---

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
