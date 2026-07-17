---
change: CHG-0050-update-current-v5-security-guidance-discovered-by-the-5-1-1-release-review
artifact: context
---

# Context

PR #389 review found that `SECURITY.md` still recommended the retired `v4` floating Action ref
while the 5.1.1 release candidate promotes the `v5` line. Historical migration and changelog
examples must remain unchanged, but current security guidance must point at `v5` and participate in
the release documentation guard.
