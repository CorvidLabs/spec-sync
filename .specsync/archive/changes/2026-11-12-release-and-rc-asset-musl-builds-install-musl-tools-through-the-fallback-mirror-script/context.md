---
change: release-and-rc-asset-musl-builds-install-musl-tools-through-the-fallback-mirror-script
artifact: context
---

# Context

On 2026-11-11 the runners' Ubuntu mirrors stopped resolving and both musl CI jobs failed at "Install musl toolchain (Linux musl)" (run 43037386037). #1061 moved the ci.yml install into `.github/scripts/install-musl-tools.sh`, which retries, then falls back to other Ubuntu mirrors. Its approved scope was ci.yml only, so it recorded that release.yml and rc-assets.yml still carried the bare `apt-get update` / `apt-get install -y musl-tools`. This change is that follow-up, prepared for the 6.3.0 release. Leif's release decision (coordinator chat, 2026-11-12, relayed by hc t370) was "yes chough 0.6.0 ships with spec-sync 6.3.0".

Why it matters: in a promote run, `promote` pushes the immutable `vX.Y.Z` tag before `build` runs. A mirror outage at the musl rows would stop `release` from publishing until someone re-dispatched (RELEASING §4 says that is idempotent). In `rc-assets.yml`, the two musl archives would be missing from the pre-release.

Decisions:

- The release.yml `build` job and the rc-assets.yml `build` job each run the script from the candidate tree they check out. That is how both already run `.github/scripts/linux-baseline.py` (#882). A candidate older than #1061 has no script, just as one older than #882 has no `linux-baseline.py`. Every 6.3.0 candidate is cut from main after #1061.
- The per-target env exports (`CARGO_TARGET_*_LINKER`, `CC_*`) stay in the workflow step unchanged. `test-linux-baseline.py` reads them there.
- Case 7 of `test-install-musl-tools.sh` covered ci.yml only, which is how the release copies were missed. It now checks all three workflows.
- No ubuntu-24.04 pin. #1065 pinned trust.yml's `trust` job because augur's prebuilt binary needs `libxml2.so.2`, which Ubuntu 26.04 dropped (CorvidLabs/augur#59). No job in release.yml or rc-assets.yml runs augur or a CorvidLabs/trust gate: `qualify` downloads only fledge v1.7.2, and the 2026-11-11 Ubuntu audit found fledge fully static. The other jobs run Python validators, Docker baseline checks, `gh` and `git`.
- The x86_64 musl rows stay on `ubuntu-latest`, as in ci.yml's `linux-baseline` job. musl output is static, and `linux-baseline.py check` runs each archive in the pinned baseline containers before upload.
