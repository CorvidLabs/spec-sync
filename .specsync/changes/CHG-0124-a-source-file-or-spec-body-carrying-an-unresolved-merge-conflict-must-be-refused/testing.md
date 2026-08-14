---
change: CHG-0124-a-source-file-or-spec-body-carrying-an-unresolved-merge-conflict-must-be-refused
artifact: testing
---

# Testing

Three directions, because this guard can fail in three ways: not firing on the
bug, firing on healthy code, or firing on this repository.

    conflicted repro   exit 1, names "HEAD contributes 'sub'; other contributes 'mul'"
    clean control      exit 0, 1 specs checked: 1 passed
    spec-sync itself   exit 0, 62 specs checked, 0 conflict mentions

The third is the acceptance test the design exists to satisfy: this repository
carries twelve well-formed conflict triples inside test string literals, two of
them in files mapped by a spec and scanned on every run.

Suite: fmt clean, clippy clean, 2238 unit + 343 integration, 0 failures.

**The trap this nearly fell into, recorded because it is generalisable.** An
earlier build of this change had both detection halves disabled — one
short-circuited by `if true { return None; }`, the other returning
`Vec::new()` unconditionally. Against that build the spec-sync-itself
acceptance test PASSED, 62 specs and zero conflict mentions, because a disabled
guard fires on nothing. The repro passed too. Only running both together
revealed the no-op. **An acceptance test that asserts an absence proves nothing
without a paired test that asserts a presence.**

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-merge-002 | The content predicate runs on a string with no repository present. `unmerged_paths` returns `None` on both failure paths, and its own test `git_discovery_failure_is_unknown_not_empty` failed until it did — the doc comment stated the rule before the code implemented it |
| REQ-exports-007 | The repro names both sides. The twelve in-repo triples do not fire, which is the discriminating evidence: detection requires declarations on both sides, and a triple inside a string literal yields none |
| REQ-validator-015 | Source and body are both refused; the body check runs before frontmatter parsing. All three read paths covered — snapshot, filesystem, and the pre-read path `issues` uses, which would otherwise keep the union bug |
| REQ-scoring-003 | API credit is withheld for a spec with a conflicted file, and withheld even when its other files parsed — scoring the readable remainder would report a confident number over a tree that cannot compile |
| REQ-cmd-merge-002 | A scan that could not run exits non-zero rather than reporting nothing needs resolution |
| REQ-cmd-diff-002 | A conflicted `files:` entry is named and excluded rather than differenced; every delta computed from the union would be fiction |
