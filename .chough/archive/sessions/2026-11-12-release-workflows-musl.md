---
kind: session
id: 2026-11-12-release-workflows-musl
aliases: [release-workflows-musl]
objective: "The release and RC-asset musl builds install musl-tools through .github/scripts/install-musl-tools.sh, as ci.yml does since #1061, so a runner mirror outage cannot stop a promoted release from publishing; release/rc jobs that run augur or a CorvidLabs/trust gate on ubuntu-latest get the ubuntu-24.04 pin (#1065, augur#59)"
modules: [github]
stage: test
stage_status: archived
iteration: 1
native_change: release-and-rc-asset-musl-builds-install-musl-tools-through-the-fallback-mirror-script
observations: [OBS-product-test-8534f714410eba1bf11854c1]
closed: 2026-11-13
outcome: done
---
# The release and RC-asset musl builds install musl-tools through .github/scripts/install-musl-tools.sh, as ci.yml does since #1061, so a runner mirror outage cannot stop a promoted release from publishing; release/rc jobs that run augur or a CorvidLabs/trust gate on ubuntu-latest get the ubuntu-24.04 pin (#1065, augur#59)

## Goal

The release and RC-asset musl builds install musl-tools through .github/scripts/install-musl-tools.sh, as ci.yml does since #1061, so a runner mirror outage cannot stop a promoted release from publishing; release/rc jobs that run augur or a CorvidLabs/trust gate on ubuntu-latest get the ubuntu-24.04 pin (#1065, augur#59)

## Where it stands

- Swapped the bare apt-get musl install for install-musl-tools.sh in release.yml build and rc-assets.yml build; extended the musl install test's workflow guard to all three workflows

## Decisions and why

- Authorization: Leif's release go (coordinator chat, 2026-11-12, relayed by hc t370): 'yes chough 0.6.0 ships with spec-sync 6.3.0'; t370 assigned this release-workflow fix as task 1 of the 6.3.0 prep. Native definition approval is Leif's own command (the classifier blocks agent approve); implementation review and ship by claude-code-t384.
- decided: no ubuntu-24.04 pin in release.yml or rc-assets.yml because no job in either runs augur or a CorvidLabs/trust gate (qualify downloads only fledge v1.7.2, which the 2026-11-11 Ubuntu audit found fully static); the pin stays only on trust.yml's trust job (#1065)
- decided: release.yml build and rc-assets.yml build run .github/scripts/install-musl-tools.sh from the checked-out candidate tree because that is how both already run linux-baseline.py (#882); a candidate older than #1061 lacks the script, as it lacks linux-baseline.py before #882
- decided: test-install-musl-tools.sh case 7 checks ci.yml, release.yml and rc-assets.yml because the guard previously covered ci.yml only, which is how the release copies were missed
- Native definition approval: Leif ran 'specsync change approve' himself (actor 0xLeif, 'definition approved (approved)'), relayed by hc t370 on 2026-11-11; the classifier blocks agent approve

## What was verified

- macOS 26, working tree on origin/main 4eed6e8d plus this change: 'fledge run musl-install-test' passed every case, and its workflow guard now finds .github/scripts/install-musl-tools.sh, with no bare 'apt-get install -y musl-tools', in ci.yml, release.yml and rc-assets.yml. Negative controls: putting back origin/main's release.yml, then rc-assets.yml, each failed the test ('<workflow> does not run .github/scripts/install-musl-tools.sh', exit 1). test-linux-baseline.py, test-validate-release-candidate.py, test-required-ci-gate.py, test-ci-trigger-paths.py, validate-workflow-runtime-pins.py and validate-release-version.py exit 0. shellcheck is clean on the test; actionlint is clean on release.yml; rc-assets.yml reports only the SC2012 info in its attach job, which origin/main also reports. ^2026-11-12-macos-26-working-tree-on-origin-main-4eed6e8d-plus-this-change

## Bugs

None.

## Open threads

## Timeline

- 2026-11-12T01:32:13Z · advance · intent → design ·  ^t-803988
  - native_change: release-and-rc-asset-musl-builds-install-musl-tools-through-the-fallback-mirror-script
  - revision: 4eed6e8dccc14ecf9fccf76b549d7e2fda8390e7
- 2026-11-12T01:32:13Z · skip · design → plan · CI workflow change only: no visuals or interactions ^t-412544
  - native_change: release-and-rc-asset-musl-builds-install-musl-tools-through-the-fallback-mirror-script
  - revision: 4eed6e8dccc14ecf9fccf76b549d7e2fda8390e7
- 2026-11-12T01:33:35Z · gate-opened · plan → plan · Native definition approval (specsync change approve) by Leif, command relayed through t370 ^t-cbd621
  - native_change: release-and-rc-asset-musl-builds-install-musl-tools-through-the-fallback-mirror-script
  - revision: 4eed6e8dccc14ecf9fccf76b549d7e2fda8390e7
- 2026-11-12T01:36:05Z · gate-closed · plan → plan · Native definition approval (specsync change approve) by Leif, command relayed through t370 ^t-090ed1
  - native_change: release-and-rc-asset-musl-builds-install-musl-tools-through-the-fallback-mirror-script
  - revision: 3f7c082ad55a97a47e7d0e397d5d3accb097bbff
- 2026-11-12T01:36:05Z · advance · plan → build · native:approved ^t-0d52a9
  - native_change: release-and-rc-asset-musl-builds-install-musl-tools-through-the-fallback-mirror-script
  - gate: native:approved
  - revision: 3f7c082ad55a97a47e7d0e397d5d3accb097bbff
- 2026-11-12T01:36:06Z · advance · build → test · native:approved ^t-6d5701
  - native_change: release-and-rc-asset-musl-builds-install-musl-tools-through-the-fallback-mirror-script
  - gate: native:approved
  - revision: 3f7c082ad55a97a47e7d0e397d5d3accb097bbff

Closed 2026-11-13: done.

## Receipts

```yaml
receipts:
  - id: 2026-11-12-macos-26-working-tree-on-origin-main-4eed6e8d-plus-this-change
    aliases: OBS-product-test-8534f714410eba1bf11854c1
    claim: "macOS 26, working tree on origin/main 4eed6e8d plus this change: 'fledge run musl-install-test' passed every case, and its workflow guard now finds .github/scripts/install-musl-tools.sh, with no bare 'apt-get install -y musl-tools', in ci.yml, release.yml and rc-assets.yml. Negative controls: putting back origin/main's release.yml, then rc-assets.yml, each failed the test ('<workflow> does not run .github/scripts/install-musl-tools.sh', exit 1). test-linux-baseline.py, test-validate-release-candidate.py, test-required-ci-gate.py, test-ci-trigger-paths.py, validate-workflow-runtime-pins.py and validate-release-version.py exit 0. shellcheck is clean on the test; actionlint is clean on release.yml; rc-assets.yml reports only the SC2012 info in its attach job, which origin/main also reports."
    result: passed
    revision: 4eed6e8dccc14ecf9fccf76b549d7e2fda8390e7
    environment: local
    content: 2aa81a2cd2b36daf7f24729a3e9eb5cb7de3089f29c1fef701ae30671881971b
    observed: 2026-11-12T01:33:33Z
```
