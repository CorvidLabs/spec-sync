---
change: CHG-0127-an-unmeasured-staleness-count-must-render-as-unknown-rather-than-zero-and-the-h
artifact: context
---

# Context

#572 and #583 both merged, and both left their sandbox gate red. That is the
whole point of the gates, and it is worth recording exactly what they caught,
because in both cases the fix was correct and the *reporting* was not.

**Gate 046 (#572).** `report` correctly exited 1 and said "Staleness
inconclusive: not a git repository". It then printed:

    Modules: 1 total, 0 stale, 1 staleness unmeasured, 0 incomplete
    "stale_modules": 0

Per-module fields were right — `stale: null`, `commits_behind: null` — but the
AGGREGATE still said zero. A dashboard scraping "N stale" reads no drift from a
run that measured none. Worse: CHG-0123's own testing.md asserted "`stale_modules`
is `null`, never `0`". That claim was written into shipped evidence and never
checked. It was false.

**Gate 047 (#583).** All fourteen pending assertions were the malformed-TOML
shape, which #583 scoped out on the grounds that `config.rs`'s hand-rolled
scanner cannot detect a parse error. True — and the gate's answer is that it
should be able to. The scanner silently skipped any line it did not recognise,
so a typo'd `[rules]` header disabled every rule while `check` reported success.

The last failing assertion was the drill declining to recognise the new refusal,
because it said "could not be parsed" where the sibling path says "could not be
loaded". The drill was right about that too.
