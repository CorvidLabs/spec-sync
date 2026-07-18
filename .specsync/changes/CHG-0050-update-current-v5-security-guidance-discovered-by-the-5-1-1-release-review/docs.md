---
change: CHG-0050-update-current-v5-security-guidance-discovered-by-the-5-1-1-release-review
artifact: docs
---

# Docs

Update `SECURITY.md` to recommend `CorvidLabs/spec-sync@v5` for current-major CI consumers while
retaining the stronger full-tag or commit guidance. The release validator includes this current
security document in its explicit Action-document allowlist; historical migration and changelog
documents remain excluded so their old-version examples stay accurate.
