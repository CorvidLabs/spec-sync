---
change: release-specsync-6-0-1-with-the-git-lfs-object-id-evidence-fix
artifact: design
---

# Design

6.0.1 publishes the evidence behavior already on `main`. A clean tracked file, including a clean Git LFS pointer, is named by its Git object id. Dirty and untracked bytes stay inside the existing 256 MiB bound. No evidence rule from pull request 800 is added.

The Action omitted-input default on this commit is the package version `6.0.1`. The annotated candidate is `v6.0.1-rc.1`. `release.yml` promotes that candidate to `v6.0.1`. The floating `v6` tag moves only after that, to this commit.
