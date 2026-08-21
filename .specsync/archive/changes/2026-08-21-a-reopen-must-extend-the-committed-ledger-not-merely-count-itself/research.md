# Research

## Defect 1 — `generation` is attacker-controlled

`approval_ledger_generation` returned `approvals.json`'s `reopenings.len()`. Whoever writes the
file writes the number, so it distinguishes nothing. Measured against a fixture built by the
`7cbe820e` binary and read back with `change status --format json`, where `forged-generation`
means the tampering plus one hand-written `ReopenRecord`:

| vector | mode | `7cbe820e` | repaired |
|---|---|---|---|
| v4 tamper-then-relocate | plain | corrupt | corrupt |
| v4 | **forged-gen** | **authenticated-history** | corrupt |
| v5 tamper+relocate, one commit | plain | corrupt | corrupt |
| v5 | **forged-gen** | **authenticated-history** | corrupt |
| v6 forged reopen/re-archive | plain | corrupt | corrupt |
| v6 | **forged-gen** | **authenticated-history** | corrupt |

So #660 closed the front door and left a window. The vectors were verified as filed; nobody
tried them with a forged generation.

## Defect 2 — the regression, bisected and instrumented

```
pre-660  (ac17bfbc)   drill 049: pass=12 fail=0 pending=0
post-660 (7cbe820e)   drill 049: pass=11 fail=1 pending=1
```

Instrumenting both binaries identically shows the bound on stages A and C **never fires** —
zero anchors, zero skips, in every call, in both builds. The regression is one added conjunct on
**stage D**, the working-tree closing-evidence fallback.

Stage D wins *every* successful transition in the pre-660 trace. Stages A and C are empty by
construction in a reopen lifecycle: acceptance is only ever reached in the working tree between
`review` and `finalize`, and never committed. Stage B can offer only the previous generation's
archived package, whose `verification.json` and `approvals.json` are — by definition of a genuine
reopen — no longer the current bytes.

At the second finalize the *new* generation's package has not been committed yet; it is what the
finalize is about to create. So "history" holds only the superseded introduction, stage B
correctly refuses it on bytes, and the sole surviving stage was switched off.

The `generation` term could not rescue this: it discriminates *among* introductions, and there is
only one.

## Why the naive revert is wrong

Deleting the conjunct returns drills 013 and 049 to green and reopens the laundering. That is
what a fake fix looks like here, so the new test is checked against a build with the conjunct
deleted as well as against the current binary.
