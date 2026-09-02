---
change: allow-the-scope-approver-to-record-scoped-review-so-solo-projects-can-ship-when-github-does-not-require-a-second
artifact: context
---

# Context

Hit on CorvidLabs/corvid-bot PR 29: GitHub required 0 approving reviews and Trust was green, but `specsync change review --reviewer leif` refused because `leif` had recorded definition approval. SpecSync invented a two-person gate the repository did not have. ADOPTING.md already names this as a solo-adopter bite. GitHub is the merge authority. Scoped review still records who signed off; it must not demand a second identity.
