---
change: CHG-0141-a-directory-named-in-files-must-score-zero-not-eighty
artifact: context
---

# Context

`score --strict` passed, at exactly 80, a spec that `check` hard-fails. Two commands over the
same spec, opposite verdicts.

    score --strict   80/100 [B], exit 0
    check            exit 1, "is a directory"

80 is the inclusive strict bar, so the spec landed precisely on the boundary and passed.

## Mechanism

A `files:` entry naming a directory was never classified as a directory. `read_to_string` on a
directory fails, the export scan returned `Unreadable`, and `Unreadable` is scored as "missing
or not UTF-8" — a message that never says the word directory. Meanwhile the freshness dimension
asked only `exists()`, and a directory exists, so `files_exist` scored 15/15.

Fifteen points for a path that exists, zero for an API it could not read, and the arithmetic
lands on 80.

So the defect is not the threshold. It is that a state the code could not interpret — "this is
not a file" — was folded into an existing category that means something else, and that category
carried a different score and a misleading message.

## Sibling sites

`validator.rs` had already been taught to reject directories; `scoring.rs` freshness, the export
scan, and `diff` had not. The fix therefore classifies once, in the export scan, and every
command that asks the question consumes that classification: validator, score, diff, issues,
lifecycle and mcp.

That is why nine source files move for a defect first reported against one command. A shared
predicate is what stops a directory being classified one way by `check` and another by `score`.

## Ruled out

Raising or lowering the strict bar. The bar is not wrong; 80 is a deliberate boundary and the
inclusive comparison is intended. A spec mapping a directory should not be scoring 80 in the
first place.
