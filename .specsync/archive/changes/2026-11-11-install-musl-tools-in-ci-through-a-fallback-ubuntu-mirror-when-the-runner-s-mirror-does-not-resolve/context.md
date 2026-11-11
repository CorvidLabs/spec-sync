---
change: install-musl-tools-in-ci-through-a-fallback-ubuntu-mirror-when-the-runner-s-mirror-does-not-resolve
artifact: context
---

# Context

Since about 14:19 UTC on 2026-11-11 both "Linux release baseline (*-musl)" jobs fail at "Install musl toolchain (Linux musl)". Run 43037386037 on main (jobs 130580209115, x86_64, ubuntu-24.04; 130580208844, aarch64, ubuntu-24.04-arm) shows apt unable to resolve every Ubuntu archive name the runners use: the x86_64 runner's own mirror list (azure.archive.ubuntu.com, archive.ubuntu.com, security.ubuntu.com) and ports.ubuntu.com on arm64. `apt-get update` only warned; `apt-get install -y musl-tools` then failed, so the Required CI gate is red on main and on every archived PR (#1052, #1045). packages.microsoft.com resolved in the same jobs.

Leif approved the fix in the coordinator chat on 2026-11-11 ("yes fix it!?", answering "can I change that CI step to retry and fall back to a different server?"), relayed by hc t370.

Decisions:

- The install moves into `.github/scripts/install-musl-tools.sh` so a stubbed apt-get can test it; the per-target env exports stay in ci.yml exactly as they were (test-linux-baseline.py reads them there).
- First attempt: the runner's own sources, with `APT::Update::Error-Mode=any` so a failed index is a failure rather than a warning. Fallback: repoint only Ubuntu archive URIs (the runner's apt-mirrors.txt mirror list or any http(s) *.ubuntu.com/ubuntu[-ports] URI) at a mirror+file list with explicit priorities; third-party sources are untouched; the fallback update has no Error-Mode so a third-party index cannot block musl-tools.
- Mirror order: mirrors outside ubuntu.com first (amd64: mirrors.edge.kernel.org, mirrors.mit.edu, mirrors.ocf.berkeley.edu; arm64: mirrors.ocf.berkeley.edu, mirrors.mit.edu, ftp.tu-chemnitz.de ubuntu-ports), the Canonical host last. archive.ubuntu.com first was rejected because it already failed inside the runner's own list. Each mirror was checked to serve noble, noble-security and musl-tools 1.2.4-2 for its architecture on 2026-11-11. apt still verifies every index against the Ubuntu archive key.
- release.yml and rc-assets.yml carry the same bare apt-get install; they are outside this change's approved scope and are reported as a follow-up.
