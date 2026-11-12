---
change: release-and-rc-asset-musl-builds-install-musl-tools-through-the-fallback-mirror-script
artifact: tasks
---

# Tasks

- [x] release.yml `build`: replace the bare apt-get musl install with `.github/scripts/install-musl-tools.sh`; keep the per-target env exports
- [x] rc-assets.yml `build`: the same swap
- [x] test-install-musl-tools.sh: the workflow guard checks ci.yml, release.yml and rc-assets.yml
- [x] Check every release.yml and rc-assets.yml job for augur or a CorvidLabs/trust gate on ubuntu-latest (none found, so no pin)
