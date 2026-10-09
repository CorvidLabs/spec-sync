---
change: release-specsync-6-0-1-with-the-git-lfs-object-id-evidence-fix
artifact: plan
---

# Plan

1. Set the package version, lockfile, Action default, CI consumer pin, and Trust pin to `6.0.1`.
2. Point README, site, adopting, migration, and security guidance at `6.0.1`, and keep the note that `@v6.0.0` still embeds `6.0.0-rc.14`.
3. Add the `6.0.1` changelog section for the object-id LFS evidence fix and move the comparison link forward.
4. Update REQ-github-002 so the current Action default and floating `v6` commit match `6.0.1`.
5. Run the release-version validator, then `change check --commit`.
