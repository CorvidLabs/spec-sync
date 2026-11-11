# Lesson bundle: install-musl-tools-in-ci-through-a-fallback-ubuntu-mirror-when-the-runner-s-mirror-does-not-resolve

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Install musl-tools in CI through a fallback Ubuntu mirror when the runner's mirror does not resolve
- **Kind**: BugFix
- **Paths**: .github/workflows/ci.yml, .github/scripts/install-musl-tools.sh, .github/scripts/test-install-musl-tools.sh, fledge.toml
- **Acceptance**: The ci.yml musl install step runs .github/scripts/install-musl-tools.sh and keeps its per-target env exports unchanged; with a stubbed apt-get whose runner mirror does not resolve, the script installs musl-tools through the fallback mirror list on the x86_64 mirror-list, arm64 ports and one-line sources layouts, leaves third-party sources alone, passes Acquire::Retries and Timeout to every apt-get call, and ends in ::error:: when every mirror fails or no Ubuntu source can be repointed; validate-action and the fledge musl-install-test task run that test

## Evidence

- Verification commit: `ff1f8d1c97e7a50f060efdd9d6535504b54803bd`
- Base commit: `ad8544e9da79725f499b38ae4d4b3a052be5f807`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

Since about 14:19 UTC on 2026-11-11 both "Linux release baseline (*-musl)" jobs fail at "Install musl toolchain (Linux musl)". Run 43037386037 on main (jobs 130580209115, x86_64, ubuntu-24.04; 130580208844, aarch64, ubuntu-24.04-arm) shows apt unable to resolve every Ubuntu archive name the runners use: the x86_64 runner's own mirror list (azure.archive.ubuntu.com, archive.ubuntu.com, security.ubuntu.com) and ports.ubuntu.com on arm64. `apt-get update` only warned; `apt-get install -y musl-tools` then failed, so the Required CI gate is red on main and on every archived PR (#1052, #1045). packages.microsoft.com resolved in the same jobs.

Leif approved the fix in the coordinator chat on 2026-11-11 ("yes fix it!?", answering "can I change that CI step to retry and fall back to a different server?"), relayed by hc t370.

Decisions:

- The install moves into `.github/scripts/install-musl-tools.sh` so a stubbed apt-get can test it; the per-target env exports stay in ci.yml exactly as they were (test-linux-baseline.py reads them there).
- First attempt: the runner's own sources, with `APT::Update::Error-Mode=any` so a failed index is a failure rather than a warning. Fallback: repoint only Ubuntu archive URIs (the runner's apt-mirrors.txt mirror list or any http(s) *.ubuntu.com/ubuntu[-ports] URI) at a mirror+file list with explicit priorities; third-party sources are untouched; the fallback update has no Error-Mode so a third-party index cannot block musl-tools.
- Mirror order: mirrors outside ubuntu.com first (amd64: mirrors.edge.kernel.org, mirrors.mit.edu, mirrors.ocf.berkeley.edu; arm64: mirrors.ocf.berkeley.edu, mirrors.mit.edu, ftp.tu-chemnitz.de ubuntu-ports), the Canonical host last. archive.ubuntu.com first was rejected because it already failed inside the runner's own list. Each mirror was checked to serve noble, noble-security and musl-tools 1.2.4-2 for its architecture on 2026-11-11. apt still verifies every index against the Ubuntu archive key.
- release.yml and rc-assets.yml carry the same bare apt-get install; they are outside this change's approved scope and are reported as a follow-up.

## From the change's testing.md

# Testing

The real DNS failure cannot be reproduced on demand, so `.github/scripts/test-install-musl-tools.sh` runs the script against stubbed sudo, dpkg and apt-get. The stub fails any Ubuntu source whose host matches the case's dead-host pattern and passes a mirror+file source when one of its hosts is live.

| Case | Expected |
|------|----------|
| x86_64 deb822 sources through the runner's apt-mirrors.txt, all ubuntu.com hosts dead | strict update fails, both stanzas repointed at the fallback list (kernel.org first, archive.ubuntu.com last), Signed-By kept, microsoft source untouched, musl-tools installed, ::warning:: |
| arm64 deb822 sources on ports.ubuntu.com, dead | repointed at the ubuntu-ports list (OCF first, ports.ubuntu.com last), installed |
| one-line sources.list (jammy) with azure and security hosts dead | all three deb lines repointed, installed |
| runner mirror healthy | exactly one update and one install, no fallback list, sources unchanged, no warning |
| every host dead, fallback included | non-zero exit, ::error:: names the fallback mirrors |
| no Ubuntu archive source to repoint | non-zero exit, ::error::, no fallback list written |
| ci.yml | runs install-musl-tools.sh; no bare apt-get install of musl-tools remains |

Every apt-get call must carry `-o Acquire::Retries=3 -o Acquire::http::Timeout=30 -o Acquire::https::Timeout=30`.

Negative controls: with the old two-line step in place of the script the test fails (x86: exited 100); without the strict first update it fails on the call order.

Also run: shellcheck on both scripts, actionlint on ci.yml, and the CI script tests that read ci.yml and fledge.toml (linux-baseline, required CI gate, trigger paths, attest and summary, clippy all targets, classify-ci-paths, workflow runtime pins). The live check is the pull request's own two musl jobs at its archived tip; the Required CI gate needs them before merge.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
