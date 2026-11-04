---
change: name-the-spec-to-edit-when-change-approve-refuses-an-unowned-affected-path
artifact: context
---

# Context

Leif asked for this in Grok Bot chat t811u on 2026-11-03, from Raven: "`change approve` refuses affected paths that no module owns (for example a new module's own `.spec.ts` files), with a fix that requires editing the module's files list; say which spec to edit in the error." The refusal listed the paths and said to "add the path to an owning spec's `files:` list" without naming the spec. In a repository with many specs the agent had to search for the right one.

The ownership rule from REQ-change-053 does not change. This change only words the refusal.
