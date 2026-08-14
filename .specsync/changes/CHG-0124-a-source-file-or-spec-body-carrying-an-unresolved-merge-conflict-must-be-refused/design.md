---
change: CHG-0124-a-source-file-or-spec-body-carrying-an-unresolved-merge-conflict-must-be-refused
artifact: design
---

# Design

Two signals, composed, because neither covers both measured cases.

**Git's unmerged list** is authoritative and costs no heuristic: a path in
`git diff --diff-filter=U` IS conflicted, whatever the extractor made of the
bytes. Zero false positives by construction. It does not fire on the confirmed
repro, whose markers were committed.

**The symptom** covers that: escalate only when the extractor reads declarations
from *both* sides of the same hunk. `conflicted_union` extracts once from the
raw content, once from each side with the hunk resolved, and reports only when
at least one symbol survives on `ours` but not `theirs` AND at least one the
other way. That is precisely what makes the union bogus, and it is why the
twelve triples in this repository do not fire: a triple inside a string literal
does not yield distinct declarations on both sides.

The evidence carries which symbols came from which side, so the error names the
mechanism rather than announcing that markers exist.

Spec bodies get the same treatment ahead of frontmatter parsing, so a conflict
inside frontmatter is named as a conflict rather than surfacing as a duplicate
key. Fenced code is blanked first — line count preserved so reported line
numbers still point at the real body — because a spec that *documents* conflict
syntax is not a spec that *contains* a conflict.

`unmerged_paths` returns `Option`: `None` is *unknown*, never *clean*. A caller
that collapses the two reports an all-clear it never verified, which is the same
fail-open being closed here. Both failure paths — git absent, git refused —
return `None`.

Three read paths exist and all three are covered: `validate_spec`'s snapshot
branch, its filesystem branch, and `specsync issues`, which pre-reads bytes and
would otherwise keep the union bug.
