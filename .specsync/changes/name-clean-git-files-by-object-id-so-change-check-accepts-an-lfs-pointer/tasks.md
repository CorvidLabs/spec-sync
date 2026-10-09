---
change: name-clean-git-files-by-object-id-so-change-check-accepts-an-lfs-pointer
artifact: tasks
---

# Tasks

- [x] Name clean Git files by object id in the workspace digest.
- [x] Accept `filter=lfs` as that pointer object and keep every other content filter fail-closed.
- [x] Keep acceptance evidence hashing the bytes of covered paths.
- [x] Cover the clean LFS pointer, a dirty rewrite, and the existing fail-closed attribute cases.
