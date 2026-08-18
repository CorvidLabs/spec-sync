# Context

`release.yml`'s `validate` job waited for a check run named `SpecSync archive binding` and
then validated it in roughly 430 lines of embedded Python. No workflow in this repository
produces that check run.

Its producer was `.github/workflows/post-merge-archive.yml`, deleted in `802ca13b` (#499). The
deletion was deliberate — #499 removed ~7,257 lines of Python, bash and YAML that re-derived
the SDD lifecycle from Git commit topology, on the grounds that SpecSync already implements it
in tested Rust that ships to users. That commit message even names a bug living in this very
file: it counted a change package's `deltas/` subdirectory as a second archive root.

**#499 removed the producer and left the consumer.**

## Measured

Check runs present on `200add8f`:

    Analyze (actions), Analyze (javascript-typescript), Analyze (rust), Build Astro Site,
    Classify changed paths, Deploy to GitHub Pages, Lifecycle gate, Lifecycle preflight,
    Packaged GitHub Action consumer, Record attestation, Required CI gate,
    SpecSync implementation ready, SpecSync scoped review, audit, coverage, fmt, site,
    spec-check, test, trust, validate-action, vscode-extension

`SpecSync archive binding` is not among them.

The wait carried no `if` and no `continue-on-error`, so `binding_status` stayed `missing`
through all twelve attempts and the job exited 1 — or hit its `timeout-minutes: 5` first.

## Why this is worse than a tag-day problem

`validate` is the only job in the lane with **no mode guard**:

| job | guard |
|---|---|
| `validate` | none |
| `qualify`, `record-qualification` | `mode == 'qualify'` |
| `authorize-release`, `promote`, `build` | `mode == 'promote'` |

The workflow's only triggers are an RC tag push (`mode=qualify`) and a dispatch
(`mode=promote`). Both run `validate`. So pushing `v6.0.0-rc.1` failed before `qualify`
started: **the internal RC was as blocked as the final tag.**

## Why it stayed invisible

The lane's first execution *is* the release. Nothing exercises `release.yml` beforehand, so a
producer deleted on 2 August produced no signal for sixteen days. The validation below the wait
reads as a working system — it parses the `external_id`, pins the app slug and id, checks the
`details_url` prefix, cross-checks the bound PR two ways — and none of it is reachable, because
the input it validates is never produced. Tested component, untested wiring.
