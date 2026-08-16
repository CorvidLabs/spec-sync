---
change: CHG-0132-a-warm-hash-cache-must-not-drop-findings-because-skipping-re-validation-without
artifact: context
---

# Context

The same command, over the same tree, disagreed with itself depending on run
history:

    first run   specs_checked: 1, warns "Undocumented export 'sub'"
    second run  {passed: true, specs_checked: 0, warnings: []}

`--force` and `--no-cache` restore the warning. So the cache is working
correctly — that is precisely WHY the findings vanish.

This is the worst of the remaining false-green defects, and not because its
blast radius is largest. It is because it makes **every other result
conditional on run history**. A green board, a clean CI run, a passing gate —
each is only as trustworthy as whether the cache happened to be cold. That
includes the drill suite policing all the other bugs.

The design question that had to be answered before writing code was which
contract the cache implements:

    Does an unchanged spec skip re-VALIDATION, or only re-EXTRACTION?

It skips re-validation. That is a legitimate optimisation — but it means the
previous verdict has to survive, and it did not. The snapshot types for storing
a per-spec result **already existed and were unused**: the mechanism was built
and never wired to the live path. So the findings were never stored, rather than
stored and not replayed, and the fix is store-then-replay rather than
replay-what-is-there.
