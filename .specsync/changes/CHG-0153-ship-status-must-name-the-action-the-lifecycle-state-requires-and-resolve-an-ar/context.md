# Context

`change ship-status` computes the correct next action and then prints a different one, and it
looks for a change's evidence somewhere the change may no longer be.

## Two defects, measured on `81f752c0`

**(a) The printed `Next:` contradicts the lifecycle.** `ship_status_report` computes
`lifecycle_next`, then `ship_next` overrides it, and the human printer renders `ship_next`.

    draft      Next: run `specsync change check <ID> --commit`, push the product tip…
               lifecycle_next: run `specsync change answer <ID> acceptance_criteria <answer>`
    approved   Next: no verification evidence recorded yet        (blockers[0], verbatim)
    archived   Next: run `specsync change check <ID> --commit`…   (on a finished change)

Obey the printed line at draft and the same binary refuses it. At approved the line is not a
command at all — it is a restatement of a blocker that already prints on its own `Blocker:`
line.

**(b) Evidence is read from a hard-coded active path.** Two reads built
`root.join(".specsync/changes").join(&record.id)` — a parallel implementation of `change_dir`
that an archived change has moved out of. So a finalized change reported `Verification: none`
and `Review: missing` for artifacts sitting in its own archive package, and rendered two
`[current]` stages at once.

## Why the report is not the whole defect

#534 names layer (a). Layer (b) is what makes the archived output wrong, and the two are the
same object seen from opposite ends: the command cannot say what to do next partly because it
cannot see what has already been done.

## The judging problem, found before writing any code

Drill 053 could not tell a real fix from a cosmetic one. Two shims built against the unfixed
tree:

| shim | drill 053 | 030 | 031 |
|---|---|---|---|
| one-line text-printer swap | `4/0/3 → 7/0/1`, three gates flip | unchanged | unchanged |
| 3-line patch asserting `done` for archived, reading no evidence | `8/0/0` **PASS** | — | — |

Both existing vacuity controls assert on the JSON field `ship_next`; the vacuous fix lives in
the text printer. They never meet. So sandbox PR #90 landed three assertions first — the
archived verification commit, the archived review, and a corrupt-archive control — red, before
this change existed. That is why the gate below is worth believing.
