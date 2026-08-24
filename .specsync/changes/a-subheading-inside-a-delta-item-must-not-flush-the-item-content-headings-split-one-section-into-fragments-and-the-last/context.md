---
change: a-subheading-inside-a-delta-item-must-not-flush-the-item-content-headings-split-one-section-into-fragments-and-the-last
artifact: context
---

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
