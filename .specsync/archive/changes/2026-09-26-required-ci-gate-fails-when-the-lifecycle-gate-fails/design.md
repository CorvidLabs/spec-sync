---
change: required-ci-gate-fails-when-the-lifecycle-gate-fails
artifact: design
---

# Design

## The gate asks each job's own question

`implementation-gate` needs every job that can finish before it: `classify`, `preflight`,
`lifecycle-gate`, the product jobs and `corvid-pet`. Its single step reads a `GATES` table with
one row per job:

```text
<job> <selected> <result>
test ${{ needs.classify.outputs.full == 'true' }} ${{ needs.test.result }}
```

`<selected>` is the job's own `if:` with any leading `always() &&` removed, or `true` for a job
with no `if:`. GitHub renders it as `true` or `false`. The step then applies:

| selected | result | verdict |
|---|---|---|
| true | success | pass |
| false | skipped | pass (classify deselected it) |
| true | skipped | **fail**: a dependency did not succeed |
| true | failure, cancelled | **fail** |
| false | anything but skipped | **fail**: the row no longer matches the job's `if:` |
| anything else | any | **fail**: malformed row |

It also keeps the previous check, so any `failure` or `cancelled` in `join(needs.*.result)` fails,
and it fails when the number of rows differs from the number of jobs in `needs`. `ci-gate` is
unchanged: it passes only when `implementation-gate` succeeds.

## Why the row repeats the condition rather than the rule living in a script

The gate evaluates the same expression, over the same fixed classify outputs, with the same
evaluator as the job's `if:`. It cannot disagree with the job unless the text differs, and the
test compares the text. A script outside the workflow would need a checkout in the gate and a
second copy of the lane rules.

## The guard

`.github/scripts/test-required-ci-gate.py` parses `ci.yml` with Psych, like the other workflow
validators, and checks:

1. Every job that does not itself depend on `implementation-gate` is in its `needs`. Any new job
   that gates on `lifecycle-gate`, or is otherwise selected before the gate, fails the test until
   it is added. `classify`, `preflight` and `lifecycle-gate` are named explicitly.
2. Every job in `needs` has exactly one row, each row reads its own job's result, and its
   selection text equals the job's `if:`. A job whose `if:` uses a status function other than a
   leading `always()` fails, because the gate cannot mirror it.
3. A simulation of the job graph, with GitHub's implicit and transitive `success()`, runs the
   gate's own step and `ci-gate`'s step under `bash -e`, as the runner does for a step that names
   no shell. It covers every classify lane, every combination of the classify flags the
   conditions read, and every event. The required gate must be green when every selected job
   succeeds, and red when any one of them fails or is cancelled.
4. The pre-#796 gate definition, kept as a fixture, reproduces the bug in the same simulation, so
   the harness can see what it guards against.

It runs in the `validate-action` CI job, which runs whenever `ci.yml` changes because a workflow
change selects the full lane, and as the Fledge task `ci-gate-test` in the `verify`, `ci` and
`repo` lanes.
