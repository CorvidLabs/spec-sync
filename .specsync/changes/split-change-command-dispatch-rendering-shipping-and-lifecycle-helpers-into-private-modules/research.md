---
change: split-change-command-dispatch-rendering-shipping-and-lifecycle-helpers-into-private-modules
artifact: research
---

# Research

## Existing behavior and evidence

The source has 4367 lines and two inline test modules. Existing cmd_change requirements pin mutation-snapshot rendering, correction-ledger privacy, degraded rosters, readiness currency, exact lifecycle staging and shipping/PR ordering. Existing focused regressions and whole-body comparison are the primary preservation evidence. The parent has already recorded pristine main full-suite sandbox Git-fixture ancestry failures; those known failures are not a reason to remove Git markers or weaken security checks.

## Independent preservation review

A separate read-only agent completed AST, import, visibility, attribute and test comparison against original main. The resulting source retains all 104 functions, 5 constants, 3 structs, one enum, one impl and 29 test functions, with unchanged bodies, literal values, assertions and attributes/doc attributes. Differences are module/import declarations, parent-private visibility/fields, formatting and four optional signature trailing commas. The sole public API remains `cmd_change`; no command helper becomes crate-visible. All 14 resulting command Rust files have explicit canonical ownership and range from 4 to 693 lines; the dispatcher is 604 lines.

Independent review found no blocker. This is agent technical review only, not the human implementation review required by the lifecycle. No scoped-review record, acceptance or archive is created.

## Focused baseline comparison

Pristine main and the split each run the same 29 leaf test names: 22 pass and the same seven fixtures fail at Git repository detection. Comparing outcomes by leaf name yields no added, removed or changed outcome. Existing test selection `commands::change::tests` remains valid through a topical umbrella, and `ship_next_action_tests` remains at its original root. The snapshot/text ledger, archive evidence and draft fixtures remain unchanged rather than being edited to mask sandbox ancestry limitations.
