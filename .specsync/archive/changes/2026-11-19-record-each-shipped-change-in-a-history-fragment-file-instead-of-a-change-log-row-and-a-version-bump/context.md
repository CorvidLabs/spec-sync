---
change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
artifact: context
---

# Context

spec-sync #1084, from the Chough 0.6.2 work. Every SpecSync-gated PR in CorvidLabs/chough conflicts in `specs/<module>/<module>.spec.md`, on the Change Log row and the `version:` bump that `change check` writes when it materializes a delta. Each merge makes the next PR need a local update and a hand-rebuilt history. The SpecSync merge driver (`*.spec.md merge=specsync`) cannot help, because GitHub's merge button and Update-branch never run merge drivers.

Leif approved the issue on 2026-11-18 ("Leif's decision (2026-11-18): approved.", written on the issue), and hc t4c relayed that to this agent (claude-code, hc t4t) as his scope approval for the issue's three bullets. Limits: no release, tag or publish, and no merge without his relayed yes.

What a session picking this up needs to know:

- Materialization writes three outputs per module: the delta, the `version:` bump and the Change Log row (`prepare_module_application`). The row was also the only evidence that a module's materialization ran (#741). The fragment takes over both roles. A row written by an earlier SpecSync still counts, so a change materialized before the upgrade is never recorded twice.
- The freeze needs no command and rewrites nothing: SpecSync stops writing the tables and `version:`, and `specsync history` reads a table as the frozen head of the module's history. Rows are never converted into fragments.
- Out of scope: two changes that both ADD requirements to one module still both append at the end of `requirements.md`, and plain git conflicts there. #1084 names the Change Log and `version:` only.
