---
change: release-and-rc-asset-musl-builds-install-musl-tools-through-the-fallback-mirror-script
artifact: testing
---

# Testing

| Check | Expected |
|-------|----------|
| `bash .github/scripts/test-install-musl-tools.sh` | `install-musl-tools ok`: the stubbed mirror-fallback cases pass, and ci.yml, release.yml and rc-assets.yml all run the script with no bare `apt-get install -y musl-tools` |
| Negative control: restore origin/main's release.yml, then rc-assets.yml, one at a time | The test fails, naming the workflow that "does not run .github/scripts/install-musl-tools.sh" |
| `python3 .github/scripts/test-linux-baseline.py` | Passes: both release workflows keep both musl targets' env exports, and the Linux rows stay in lockstep |
| `test-validate-release-candidate.py`, `test-required-ci-gate.py`, `test-ci-trigger-paths.py`, `validate-workflow-runtime-pins.py`, `validate-release-version.py` | Pass unchanged |
| `shellcheck .github/scripts/test-install-musl-tools.sh`; `actionlint` on both workflows | Clean, apart from the SC2012 info in rc-assets.yml's `attach` job, which origin/main already reports |

The release workflows run only on an RC tag push or a dispatch, so this PR cannot run them. Their first live run is the 6.3.0 candidate. The script itself runs live in this PR's own CI musl jobs.
