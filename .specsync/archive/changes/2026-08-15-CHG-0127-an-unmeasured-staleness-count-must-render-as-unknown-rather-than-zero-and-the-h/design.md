---
change: CHG-0127-an-unmeasured-staleness-count-must-render-as-unknown-rather-than-zero-and-the-h
artifact: design
---

# Design

**The stale count.** `0` is an answer. When every module's staleness was
unmeasurable there is no answer, so text renders `stale unknown` and JSON
renders `null`. A number appears only when at least one module was actually
measured — the mixed case keeps its count, since some modules genuinely were
measured and the unmeasured ones are already reported separately.

**The scanner.** An unterminated header is recorded as a load error. The test is
deliberately narrow: a line that opens `[` and never closes it is unambiguous,
whereas "anything this scanner cannot parse" would reject valid TOML the scanner
simply does not implement — multi-line strings, inline tables. Array
continuation lines never reach the check; they are consumed by the multi-line
array branch.

**The wording.** Both shapes now say "config file exists but could not be
loaded; built-in defaults are in use", with the specific cause following. Two
shapes of one failure should read the same way, and a consumer matching on a
refusal should not have to know which door produced it. That the drill matched
on a single phrase is a feature: it is what made the divergence visible.
