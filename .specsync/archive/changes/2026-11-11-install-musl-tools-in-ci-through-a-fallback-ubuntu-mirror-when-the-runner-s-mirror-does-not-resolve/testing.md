---
change: install-musl-tools-in-ci-through-a-fallback-ubuntu-mirror-when-the-runner-s-mirror-does-not-resolve
artifact: testing
---

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
