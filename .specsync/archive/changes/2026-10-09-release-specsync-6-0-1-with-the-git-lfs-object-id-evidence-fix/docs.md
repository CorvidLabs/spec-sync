---
change: release-specsync-6-0-1-with-the-git-lfs-object-id-evidence-fix
artifact: docs
---

# Docs

The changelog `6.0.1` section states that `change check` names a clean Git file by its object id and accepts a `filter=lfs` pointer when the driver is Git LFS. README, the site, `MIGRATION.md`, `docs/ADOPTING.md`, and `SECURITY.md` pin `6.0.1`. Each one still says that `@v6.0.0` embeds default `6.0.0-rc.14`.
