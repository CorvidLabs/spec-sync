# Lesson bundle — pin-the-trust-gate-to-stable-1-2-0

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Pin the Trust gate to stable 1.2.0
- **Kind**: Operations
- **Paths**: .github/workflows/trust.yml
- **Acceptance**: The Trust gate step in .github/workflows/trust.yml resolves CorvidLabs/trust@fcc889f54d8b4892a81af463c5a0250e2be66fc5 (v1.2.0, the peeled stable release commit) instead of e0272543 (v1.2.0-rc.4). The runner-local file:// SpecSync mirror still works unchanged: Trust downloads specsync-linux-x86_64.tar.gz and its .sha256 from specsync-download-base-url, revalidates the checksum, and gates on the pull request's own freshly built binary rather than a published release. specsync-version stays 6.0.0. The hosted trust check passes.

## Evidence

- Verification commit: `7792f4e38bb030882c43dc0b98e7cae2f8e59805`
- Base commit: `15e53cab4ea1271f6c566f8372f4b584c94a07b8`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

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

## From the change's testing.md

# Testing

There is no unit test for a CI pin. Verification is a source comparison of the
two action revisions plus the hosted run itself.

## Verified before the ref was changed

Both revisions of `scripts/trust_cli.py` were read directly from the Trust
repository at `e0272543` (v1.2.0-rc.4) and `fcc889f5` (v1.2.0):

- `specsync_asset_urls(version, download_base_url, os_name, arch)` is
  byte-identical across the two revisions. When `download_base_url` is set it
  returns `{base}/specsync-{os}-{arch}.tar.gz` and `{base}/...tar.gz.sha256`,
  and only falls back to the GitHub releases URL when that input is empty.
- `download_bytes(url)` at 1.2.0 still branches on `url.startswith("file:")`
  and reads the asset from the local path.
- `specsync-download-base-url` is still a declared input of the 1.2.0 action.

Together these establish that the runner-local mirror path is unchanged, so the
gate still exercises the pull request's own binary.

## Verified locally on the committed tree

- `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/trust.yml'))"`
  exits 0, so the workflow still parses.
- `git diff` against the base branch is one changed line in one file.

## Verified on the hosted run

The pull request's own `trust` check is the real test. A passing run proves in
one step that Trust 1.2.0 resolved, that it installed SpecSync 6.0.0 from the
`file://` mirror, that the checksum revalidation accepted the locally built
archive, and that the contract, risk, and provenance stages behaved as before.

Note that both `trust` and `Lifecycle gate` run
`specsync change audit --strict` before the gate itself, which is why this
change record exists: `.github/workflows/trust.yml` is a meaningful path and
the audit fails closed when no active change covers it.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
