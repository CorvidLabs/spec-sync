---
change: CHG-0127-an-unmeasured-staleness-count-must-render-as-unknown-rather-than-zero-and-the-h
artifact: testing
---

# Testing

The evidence is the gate board, before and after, on merged main:

    gate  before                          after
    046   pass=7  fail=0 pending=4        pass=11 fail=0 pending=0
    047   pass=60 fail=0 pending=14       pass=74 fail=0 pending=0

Both directions, measured by hand as well:

    no git      text "1 total, stale unknown, 1 staleness unmeasured"   stale_modules: null
    healthy git text "1 total, 0 stale, 0 incomplete"                   stale_modules: 0

The right-hand row is what separates this from suppressing the count entirely.

Suite: fmt clean, clippy clean, 2242 unit + 367 integration, 0 failures.

**A test assertion was changed, which normally means a fix is being bent to fit.**
`report_json_never_states_a_staleness_it_could_not_measure` asserted
`stale_modules == 0`, reasoning "an unmeasured module is not a stale one". True
of the module, wrong of the count — and in direct contradiction of the test's own
name. Drill 046, written independently, disagreed with it. An independent check
disagreeing with a test is what makes changing the test legitimate rather than
convenient, and the reasoning is left in a comment at the call site.

That is now the third place a check encoded the buggy behaviour as correct, after
sandbox drills 037 and 034. All three would have made a correct fix look like a
regression.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-cmd-report-004 | Gate 046 goes 4 pending to 0. The healthy-git control still prints `0 stale` and `stale_modules: 0`, so the count was made honest rather than removed |
| REQ-config-011 | Gate 047 goes 14 pending to 0. The final assertion needed the shared refusal wording, not new behaviour — which is the drill catching a divergence between two messages for one failure |
