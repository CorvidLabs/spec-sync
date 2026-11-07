---
change: keep-a-clean-merge-driver-result-faithful-to-both-sides-quote-yaml-typed-list-items-end-the-change-log-at-its-first
artifact: tasks
---

# Tasks

- [x] Red tests for #870, #872, #875 and #876, failing on main (committed with the draft)
- [x] #870: checked increment for change-log `Version` renumbering, conflict at `u64::MAX`
- [x] #872: quote YAML-typed list items when rendering a union
- [x] #875: the first blank line after the change-log rows ends the table
- [x] #876: three-way merge of the version decoration, same result in either order
- [x] Mutants: undoing each fix fails its test
- [x] Specs (merge invariants 22-23, Change Log, 3md lesson, tasks), CHANGELOG

After `change check --commit` (plain text, not tasks): `fledge lanes run verify` and `fledge lanes run pre-push` on the pinned toolchain.

Human gates (plain text, not tasks): scope approval and implementation review come from Leif through the orc (t18).
