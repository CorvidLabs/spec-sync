---
change: CHG-0112-a-tree-with-source-and-no-specs-must-show-its-coverage-number-and-must-not-pass
artifact: context
---

# Context

## What led here

CorvidLabs/spec-sync#560, found by **sweeping** for affirmative claims rather than by a bug
report. A project with real source and an empty `specs/`:

```
$ specsync check --strict
No spec files found in .../specs/. Run `specsync generate` to scaffold specs.
exit 0

$ specsync coverage
File coverage: 0/1 (0%)
LOC coverage:  0/3 (0%)
exit 0
```

`check` prints the coverage footer in every other path. Here it was omitted entirely, so a
CI log carried no figure at all.

## The code already intended otherwise

The branch handling this carries a comment stating the requirement outright:

> No specs to validate — but a requested gate must still be evaluated against source
> coverage. Otherwise `check --require-coverage N`, `--enforcement enforce-new`, or
> `--strict` silently PASS in exactly the state they exist to catch: a project with source
> code but no specs.

`--require-coverage` and `--enforcement enforce-new` do gate. **`--strict` did not**, because
`compute_exit_code` escalates *warnings* under `--strict` and a project with no specs
produces none. The code did not do what its own comment said.

That is worth recording: the requirement was understood and written down, and the
implementation still did not meet it. Reading the comment and believing it would have
closed this as already-handled.

## Why it matters

This is the state immediately after `specsync init`, and the state of any repository whose
`specs/` was deleted or mis-configured. A caller who asked for strict validation of a tree
that was never measured was told it was clean.

## What a session picking this up needs to know

The gate is deliberately confined to trees that **have source**. An empty project, or one
whose specs simply have not been generated yet, must still exit 0 — otherwise `check`
becomes unusable as the first command a new project runs, which is the defect
CHG-0107 fixed.

This is the sixth instance of one class this cycle (#546, #547, #548, #550, #553, #560):
a category empty for want of *input*, read as want of *problems*.
