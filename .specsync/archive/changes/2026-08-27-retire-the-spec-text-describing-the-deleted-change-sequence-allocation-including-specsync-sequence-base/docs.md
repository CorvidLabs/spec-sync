---
change: retire-the-spec-text-describing-the-deleted-change-sequence-allocation-including-specsync-sequence-base
artifact: docs
---

# Docs

`AGENTS.md` carried a "Multi-clone / multi-agent sequence IDs" section telling agents to set
`SPECSYNC_SEQUENCE_BASE=<N>` per agent with disjoint ranges, and to expect `change new` to
floor on the remote default branch's ledger after a fetch. Neither has had an implementation
since #665. The guidance was inert rather than dangerous — the collision it guarded against
cannot occur once identity is a slug — but it instructed agents to do something that does
nothing, which is exactly the drift this tool exists to catch.

It is replaced by "Multi-clone / multi-agent change identity", which states what is true now:

- identity is a slug minted from the description, so two clones can no longer mint the same
  number by failing to see each other;
- `change new` refuses a slug already in use and names the existing change, its location and
  its state;
- two clones that independently choose the same description do collide at merge, and the way
  out is a distinct description, not a renumber, because there is no number;
- `.specsync/change-sequence.json` is read-only history that must never be recorded
  downwards, with the restore command spelled out;
- the historical ordinals it carries still take part in collision accounting, so an
  acknowledged historical collision stays acknowledged.

No other document, help text, README, site page or example mentions `SPECSYNC_SEQUENCE_BASE`.
`docs/6-0-findings.md` and `docs/SESSION-SUMMARY-6-0.md` mention multi-clone drills, but as
records of past investigation rather than live guidance, and are left as history.
