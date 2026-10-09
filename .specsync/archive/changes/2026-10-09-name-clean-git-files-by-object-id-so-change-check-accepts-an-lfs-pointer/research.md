---
change: name-clean-git-files-by-object-id-so-change-check-accepts-an-lfs-pointer
artifact: research
---

# Research

Published SpecSync 6.0.0 fails `change check` on `Git filter attribute is not supported for canonical evidence` when any tracked path has `filter=lfs`. The failing path can be outside the change. `specsync check` does not compute this digest, so product spec-check and trust can stay green.

Allowing `filter=lfs` and then loading every clean blob is not enough. In the repository that hit this, unique index blobs summed to about 365 MiB and the per-path sum was about 513 MiB, mostly tracked `test-results/`. The 256 MiB cap is over bytes that are read. The LFS archive itself is a pointer of about 134 bytes in Git. Its smudged working tree is not the object `change check` should name when the file is clean.

Git already identifies a clean blob by object id. Hashing path, kind, mode, and that id keeps the digest deterministic. A worktree edit Git sees as modified is still hashed as bytes, so a real edit still changes the digest. A dirty LFS smudge larger than 256 MiB can still hit the cap. A clean one does not.

Raising the cap, disabling LFS, or untracking another project's files does not fix the product. Those stay out of scope.
