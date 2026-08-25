---
change: a-subheading-inside-a-delta-item-must-not-flush-the-item-content-headings-split-one-section-into-fragments-and-the-last
artifact: testing
---

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
