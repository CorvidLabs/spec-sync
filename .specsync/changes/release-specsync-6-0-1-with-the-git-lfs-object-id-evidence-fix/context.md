---
change: release-specsync-6-0-1-with-the-git-lfs-object-id-evidence-fix
artifact: context
---

# Context

`main` at `495f2536` already names a clean Git file by its object id (`specsync.project-input-digest.v4`) and accepts `filter=lfs` only for the Git LFS driver and a pointer blob. The published binary is still 6.0.0, so Peck's installed SpecSync still rejects that attribute.

This change is the 6.0.1 distribution bump on that commit. It does not merge the draft in pull request 800. The immutable `@v6.0.0` Action tag stays on its existing commit and still embeds default `6.0.0-rc.14`.

The release candidate tag is cut only after this commit is on `origin/main`. Promotion and the floating `v6` move happen after that candidate qualifies.
