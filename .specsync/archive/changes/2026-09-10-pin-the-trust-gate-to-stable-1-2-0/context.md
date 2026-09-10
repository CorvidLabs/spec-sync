---
change: pin-the-trust-gate-to-stable-1-2-0
artifact: context
---

# Context

## What led here

Trust 1.2.0 and SpecSync 6.0.0 were released as stable on 2026-09-09. An
organization-wide rollout then moved 176 CorvidLabs repositories onto
`CorvidLabs/trust@fcc889f54d8b4892a81af463c5a0250e2be66fc5` (`v1.2.0`) with
SpecSync resolving to 6.0.0.

An audit of every default branch afterwards found this repository still pinned
its own Trust gate to `e0272543ad5c88ec75cea1e606cc2efa552af25b`, the tag object
for the `v1.2.0-rc.4` prerelease. That left the repository that publishes
SpecSync 6.0.0 as the only one in the organization gated by a Trust release
candidate, while its consumers ran the stable release.

`specsync-version` in this workflow was already `"6.0.0"`, so only the Trust
action ref was stale.

## Constraints a session picking this up needs to know

The Trust step here is not a normal consumer pin. This workflow builds the pull
request's own SpecSync binary with `cargo build --release --locked`, packages it
as a checksum-protected runner-local mirror under `${RUNNER_TEMP}`, and passes
`specsync-download-base-url: file://${{ runner.temp }}/specsync-trust-mirror`.
Trust therefore gates on the binary built from the pull request rather than on a
published GitHub release. Any change to the Trust ref has to preserve that.

That mechanism was verified against the 1.2.0 action before the ref was changed:

- `specsync_asset_urls()` in `scripts/trust_cli.py` is byte-identical between
  `e0272543` and `fcc889f5`. It still derives `{base}/specsync-{os}-{arch}.tar.gz`
  and the matching `.sha256` from `download_base_url` whenever that input is set.
- `download_bytes()` at 1.2.0 still special-cases `file:` URLs and reads them
  from disk instead of over the network.
- `specsync-download-base-url` is still a declared input on the 1.2.0 action.

## Ruled out

Changing `specsync-version`, the mirror packaging steps, or the
`specsync change audit --strict` preflight. None of them are stale and none are
implicated in the drift this change corrects.

Rewriting `docs/RELEASING.md`. Its note that consumers still on `6.0.0-rc.12` or
Trust `v1.2.0-rc.4` should move to Trust 1.2.0 is accurate migration guidance,
not drift, so it is left as written.
