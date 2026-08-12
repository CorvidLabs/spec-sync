---
change: CHG-0106-make-verification-currency-a-content-question-delete-the-git-ancestry-walk-the
artifact: docs
---

# Docs

`CHANGELOG.md` records three user-visible consequences: squash merges no longer orphan
verification evidence, the lifecycle no longer instructs a commit that its own gate refuses,
and change-then-revert no longer stales evidence. The third is a reduction in detection and is
stated as such rather than implied.
