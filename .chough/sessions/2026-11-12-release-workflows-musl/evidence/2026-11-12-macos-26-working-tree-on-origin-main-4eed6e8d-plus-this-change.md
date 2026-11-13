---
kind: evidence
id: 2026-11-12-macos-26-working-tree-on-origin-main-4eed6e8d-plus-this-change
aliases: [OBS-product-test-8534f714410eba1bf11854c1]
claim: "macOS 26, working tree on origin/main 4eed6e8d plus this change: 'fledge run musl-install-test' passed every case, and its workflow guard now finds .github/scripts/install-musl-tools.sh, with no bare 'apt-get install -y musl-tools', in ci.yml, release.yml and rc-assets.yml. Negative controls: putting back origin/main's release.yml, then rc-assets.yml, each failed the test ('<workflow> does not run .github/scripts/install-musl-tools.sh', exit 1). test-linux-baseline.py, test-validate-release-candidate.py, test-required-ci-gate.py, test-ci-trigger-paths.py, validate-workflow-runtime-pins.py and validate-release-version.py exit 0. shellcheck is clean on the test; actionlint is clean on release.yml; rc-assets.yml reports only the SC2012 info in its attach job, which origin/main also reports."
result: passed
evidence_kind: product-test
method: automated
revision: 4eed6e8dccc14ecf9fccf76b549d7e2fda8390e7
content: 2aa81a2cd2b36daf7f24729a3e9eb5cb7de3089f29c1fef701ae30671881971b
environment: local
criteria: [CI-7, INSTALL-1.c]
requirements: [REQ-github-025]
contracts: [specs/github/github.spec.md]
limitations:
  - release.yml and rc-assets.yml run only on an RC tag push or a dispatch, so the swapped step's first live run is the 6.3.0 candidate; the script itself runs live in this PR's own CI musl jobs
  - "Stubbed apt-get: the real mirror fallback is exercised only on hosted runners"
observed: 2026-11-12T01:33:33Z
---
**macOS 26, working tree on origin/main 4eed6e8d plus this change: 'fledge run musl-install-test' passed every case, and its workflow guard now finds .github/scripts/install-musl-tools.sh, with no bare 'apt-get install -y musl-tools', in ci.yml, release.yml and rc-assets.yml. Negative controls: putting back origin/main's release.yml, then rc-assets.yml, each failed the test ('<workflow> does not run .github/scripts/install-musl-tools.sh', exit 1). test-linux-baseline.py, test-validate-release-candidate.py, test-required-ci-gate.py, test-ci-trigger-paths.py, validate-workflow-runtime-pins.py and validate-release-version.py exit 0. shellcheck is clean on the test; actionlint is clean on release.yml; rc-assets.yml reports only the SC2012 info in its attach job, which origin/main also reports.**

Recorded as passed in local at revision 4eed6e8dccc1, observed 2026-11-12T01:33:33Z.
