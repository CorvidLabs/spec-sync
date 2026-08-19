# Plan

## Layer (b): resolve evidence where the change actually is

Reuse the existing resolver rather than adding a third. `find_change_dir` already answers
active-or-archive, already rejects an ambiguous id, and already has ~20 call sites. It was
private to `change.rs`, so `commands/change.rs` had grown its own path construction instead —
which is the defect.

Make it `pub`, resolve once into `evidence_dir`, and use it for both artifacts. This **removes**
two parallel implementations of `change_dir` rather than adding a fourth idiom, which is the
opposite of what this release keeps getting bitten by.

Fall back to the active path when resolution fails. A status command must still render; a
malformed id should not make inspection impossible.

## Layer (b) reads leniently, deliberately

`.ok()` on both `fs::read_to_string` and `serde_json::from_str`. A strict `?` here turns
`ship-status` and `ship` from rc=0 into rc=1 on a repository whose archived evidence is already
damaged — measured. The fix for an inspection command must not be the thing that breaks
inspection. Unreadable evidence degrades to "none recorded".

## Layer (a): the lane may narrow, never contradict

Same rule as the CI lane classification in #626. Outside the shipping window — Draft, Accepted,
Archived — the ship lane's advice is premature or spent, so `lifecycle_next` wins outright.

And the blocker arm goes away entirely. A blocker says what is wrong, not what to do; blockers
already render on their own lines. My first attempt kept the blocker as a suffix
(`{action} — blocked: {blocker}`) and drill 053's gate correctly rejected it: the gate matches
the blocker text as a substring, and it is right to, because a `Next:` line's whole job is to be
a command someone can run.

## Not in scope

The same "printed command the binary refuses" family appears five more times —
`src/change.rs:14546, :14575, :14600, :14704, :14713` print ``run `specsync change reopen {}` ``
without the required `--actor`/`--reason`, failing rc=2 if pasted. Recorded on #433, not fixed
here.
