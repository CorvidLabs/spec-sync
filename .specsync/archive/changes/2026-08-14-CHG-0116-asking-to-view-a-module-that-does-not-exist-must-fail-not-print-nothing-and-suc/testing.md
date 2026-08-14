---
change: CHG-0116-asking-to-view-a-module-that-does-not-exist-must-fail-not-print-nothing-and-suc
artifact: testing
---

# Testing

## Strategy

A fix that makes a command fail is easy to over-apply. The controls exist to prove the new
failure is confined to "the filter matched nothing" and has not leaked into the paths that
must keep working.

## Verified by hand

| case | before | after |
|---|---|---|
| `--spec no_such_module` | 0 bytes, exit 0 | `error: no spec module named \`no_such_module\`` + `available: alpha`, exit 1 |
| `--spec alph` (near miss) | 0 bytes, exit 0 | `did you mean: alpha`, exit 1 |
| **control** — `--spec alpha` (exists) | renders | unchanged, renders, exit 0 |
| **control** — no filter at all | renders | unchanged, renders, exit 0 |

The last control is the load-bearing one. A fix keying on "a filter was given but nothing
matched" is one edit away from breaking the unfiltered path, which renders every spec and
must not be treated as a filter that matched nothing.

## Regression surface

2210 unit and 331 integration tests pass unchanged. The change adds counters and a terminal
branch; the rendering path itself is untouched.

## Not covered

No unit test asserts the new wording or the suggestion logic. `cmd_view` has no output-test
harness in this change's scope. Behavioural pinning belongs in the sandbox — and `view` has
no drill at all today, which is worth its own entry in the working queue.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-cmd-view-002 | `cargo test` (2210 + 331, 0 failures) plus the four cases above: an unknown module and a near-miss both exit 1 with the name and a suggestion, while an existing module and the unfiltered run are byte-for-byte unchanged at exit 0 — which confines the new failure to a filter that matched nothing |
