---
change: describe-the-cli-recursion-guard-as-refusing-an-inherited-verification-context-not-a-specsync-made-child
artifact: context
---

# Context

Issue #911, split out of the #899 review (Codex P2). #899 (#848, audit D4) reconciles the change and
cmd_comment contracts with the runtime: `change check` and `comment` spawn no configured verification
command (REQ-change-023, REQ-change-091). Its review found the same promise in the cli module, which was
outside its approved specs:

- REQ-cli-003: "The root CLI dispatcher SHALL fail closed when a configured verification child re-enters
  lifecycle checking or mutation."
- The cli spec Purpose and the `cli.3md` context plane: "blocks configured verification children from
  recursively dispatching ...".
- docs/HLD.md, the failure-mode table: "Recursive calls (a verification child calling `check` or
  `change`) are refused".

The guard (`reject_recursive_lifecycle_dispatch` in src/main.rs) is live and correct. Only the wording
implies that SpecSync makes such a child.

Intent: SHIP-1.c.1 and SHIP-1.c.2, which Leif approved on 2026-11-05 at 20:24 MST (relayed by orc t18).
They are captured here with byte-identical text to #899, so the two branches merge cleanly. No runtime
change. The orc asked for this to go up to "ready for approval" only.
