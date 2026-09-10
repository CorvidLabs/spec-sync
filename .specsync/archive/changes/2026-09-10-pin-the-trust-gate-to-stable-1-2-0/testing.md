---
change: pin-the-trust-gate-to-stable-1-2-0
artifact: testing
---

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
