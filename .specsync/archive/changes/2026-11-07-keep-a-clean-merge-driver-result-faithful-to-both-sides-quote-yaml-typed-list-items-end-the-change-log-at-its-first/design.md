---
change: keep-a-clean-merge-driver-result-faithful-to-both-sides-quote-yaml-typed-list-items-end-the-change-log-at-its-first
artifact: design
---

# Design

All changes are in `src/merge_driver.rs`; no public API changes.

- **#872, `render_list_item`.** Besides today's rule (plain only for safe path characters), it double-quotes an item YAML would read as something other than a string:
  - `true/false/yes/no/on/off/y/n/null/~` in any case;
  - anything Rust parses as `f64` (integers, floats, exponents, `inf`, `nan`, a leading sign);
  - `.inf` / `.nan` with an optional sign;
  - `0x`/`0o`/`0b` integers;
  - digit-and-underscore numbers such as `1_000` (YAML 1.1).
  Over-quoting is always safe because the value stays a string. These three fields (`files`, `db_tables`, `depends_on`) are string lists, so a plain `123` written quoted gets its intended type.
- **#875, `parse_changelog_table`.** The first blank line after the separator ends the rows, and it and everything after it become the suffix. Today an interior blank is skipped unless only blanks follow. The suffix is already merged as a text region, so text after the table merges like any other text: a one-sided edit is clean, and overlapping two-sided edits conflict.
- **#876, `merge_version_field`.** The decoration after the number merges three ways (see the rules below). The decoration is the text after the closing quote, or from ` #` for a plain scalar. If the merge conflicts, the field is a conflict.
  - The rendered line takes its decoration from the side whose decoration won.
  - The early `Keep(theirs)` shortcut applies only when that side's decoration is the merged one, so a one-sided edit is still returned byte for byte.
  - Quote style divergence stays a conflict, as today (invariant 22).
- **Three-way decoration rules (#876):**
  - Both sides equal: keep it.
  - One side kept the ancestor's: take the other side's.
  - Both changed it differently: conflict.
- **#870, `union_changelog_tables`.** `next = next.checked_add(1)`; on `None` it returns `changelog_conflict(ours, theirs)`, the same conflict as a non-integer cell.
- Rejected: an explicit merge of two different comments (for example `# A; B`). It invents text neither side wrote, and MERGE-5 prefers a conflict to a guess.
- Rejected: quoting every list item after a union. It churns every spec's formatting on each merge.
