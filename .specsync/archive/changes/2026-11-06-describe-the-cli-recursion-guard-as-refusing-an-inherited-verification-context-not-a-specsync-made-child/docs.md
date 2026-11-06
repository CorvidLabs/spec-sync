---
change: describe-the-cli-recursion-guard-as-refusing-an-inherited-verification-context-not-a-specsync-made-child
artifact: docs
---

# Docs

docs/HLD.md, failure-mode table: the "Two lifecycle commands at once" row now says that a `check` or
`change` inheriting `SPECSYNC_VERIFICATION_CONTEXT` from a caller is refused, and that SpecSync itself
starts no verification child. `CHANGELOG.md` gets one Unreleased entry. No site page describes the guard.
