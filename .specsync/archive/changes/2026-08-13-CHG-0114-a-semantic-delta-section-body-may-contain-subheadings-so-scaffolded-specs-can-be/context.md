---
change: CHG-0114-a-semantic-delta-section-body-may-contain-subheadings-so-scaffolded-specs-can-be
artifact: context
---

# Context

## What led here

CorvidLabs/spec-sync#564. The tool generated specs whose sections its own lifecycle refused
to accept.

`specsync scaffold` writes `### Structs & Enums`, `### Traits`, and `### Functions` inside
`## Public API`, and `### Consumes` / `### Consumed By` inside `## Dependencies`. The delta
grammar treated **every** `###` line as an item heading, so:

```
$ specsync change approve <ID> --actor tester
error: invalid delta item heading `### Structs & Enums`
exit 1
```

Nothing was hand-edited. Scaffold a module, try to change its public contract through the
lifecycle, and `approve` refuses the section the tool had just written.

## Why it was found late

It presents as a papercut. It had been hit five times in this repository — on the
`generator`, `hash_cache`, `comment`, `cli`, and `git_utils` specs — and each time the
quickest way forward was to convert that one spec's subheadings to bold and move on. Five
instances is a pattern, and asking whether the *generator* emits the un-deltaable form is
what turned five annoyances into one defect.

## A correction worth recording

The first diagnosis was wrong. `src/generator.rs:136` emits `### Exported Functions`, and
that was nearly filed as the cause. The path `scaffold` actually takes emits different
labels — `### Structs & Enums`, `### Traits`, `### Functions` — so a report written from
reading the template would not have matched what anyone reproduces. The end-to-end run is
what produced an accurate issue.

## Why the parser, not the generator

Changing `scaffold` to emit bold labels fixes **new** projects and leaves every existing one
broken; this repository alone had five specs in that state.

The parser is also where the mistake is. The grammar identifies its own items by keyword —
`REQUIREMENT ` and `SPEC SECTION ` — so heading depth was never what distinguished them.
Rejecting every other `###` was an over-broad guard, not a design requirement.
