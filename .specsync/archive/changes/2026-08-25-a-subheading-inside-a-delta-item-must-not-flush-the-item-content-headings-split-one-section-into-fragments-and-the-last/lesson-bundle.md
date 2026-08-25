# Lesson bundle — a-subheading-inside-a-delta-item-must-not-flush-the-item-content-headings-split-one-section-into-fragments-and-the-last

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: A subheading inside a delta item must not flush the item: content headings split one section into fragments and the last one wins
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change.rs, src/change_tests.rs, specs/change/change.spec.md
- **Acceptance**: a delta section carrying content subheadings parses as ONE item holding all of them, not one item per subheading
- **Acceptance**: a real item heading still ends the previous item, so distinct sections are not merged
- **Acceptance**: a delta declaring the same operation, target and key twice is refused rather than silently keeping the last
- **Acceptance**: the living spec no longer loses documented behaviour a change never touched

## Evidence

- Verification commit: `4e379341b822762b7d1196ac551f9b3ccaf9858f`
- Base commit: `875752ee991d458db172dec6ceb712462fe2a614`
- Verified by: `cargo test change::`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

# Context

A delta modifying a spec section that contains `###` subheadings silently dropped everything
above the final subheading. It happened during #697: a delta declaring five Behavioral Examples
scenarios produced a living spec containing one, deleting three pre-existing documented contracts
— `change new --json`, `reopen`'s audit record, and `finalize` — with exit 0.

Filed as #699 with the wrong mechanism. That issue says `### Scenario` collides with the delta
format's `### SPEC SECTION` level and is therefore unrepresentable, and proposed failing closed on
such deltas. Reading the parser shows otherwise, and the real cause is smaller and fixable.

## The actual mechanism

`parse_delta` calls `flush(...)` at the TOP of the `### ` branch, before deciding whether the
heading is an item heading or content:

    if let Some(header) = line.strip_prefix("### ") {
        flush(...);                       // ends the current item unconditionally
        let (target, key) = if ... REQUIREMENT ... {
        } else if ... SPEC SECTION ... {
        } else if current_target.is_some() {
            body.push(line.to_string());  // ... but body was just cleared
            continue;

`flush` pushes an item and clears the body. So every content subheading ended the item and started
a fresh body under the SAME key. One `### SPEC SECTION Behavioral Examples` with three scenarios
became three items keyed `Behavioral Examples`, each holding one fragment, and application kept the
last.

#564 already taught this grammar that a `###` inside an open item is content — it added exactly
that branch, and the comment above it explains why. It fixed the classification and left the flush
above it. Half a fix, which is why the symptom looked like a grammar limitation rather than an
ordering bug.

## Why the target file's style decided the outcome

`change.spec.md` uses `**Scenario:**` bold, so its section survived intact. `cmd_change.spec.md`
uses `### Scenario`, so it lost content. 59 of 62 spec files use the vulnerable style, and the
corruption depended on the file being edited rather than on anything in the delta.

## From the change's testing.md

# Testing

`a_content_subheading_does_not_split_a_delta_item` — the regression, using the shape that did the
damage: three scenarios under one section. Old code yields three items; the fix yields one holding
all three.

`a_real_item_heading_still_ends_the_previous_item` — **honest label: this is the CONTROL.** Real
item headings must still terminate the previous item, or the fix would merge distinct sections.

`a_duplicated_section_key_is_refused_rather_than_overwritten` — the other route into the same
silent loss, now fail-closed with a message naming what would have been discarded.

## Discrimination

Measured in the wild rather than asserted: under the old parser, `cmd_change.spec.md` lost two of
three scenarios during #697, and the loss was only noticed because an independent reviewer diffed
the applied result against the delta. That is the failure these tests now prevent.

## Where these lessons go

- `specs/change/context.md`
