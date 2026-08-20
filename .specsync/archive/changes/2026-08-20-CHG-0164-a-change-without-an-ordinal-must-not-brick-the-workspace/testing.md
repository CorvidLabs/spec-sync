---
change: CHG-0164-a-change-without-an-ordinal-must-not-brick-the-workspace
artifact: testing
---

# Testing

| Test | Discriminates | Proves |
|---|---|---|
| `a_workspace_holding_an_ordinal_free_change_still_enumerates` | yes | enumeration survives; the numbered change is still listed for collision detection and the ordinal-free one is simply absent |
| `two_changes_claiming_one_ordinal_are_still_refused` | control | collision detection is intact, so skipping was not achieved by skipping everything |

Reproduced end to end before fixing, in a scratch repository with one hand-made slug-only
directory:

```
before   error: invalid change ID `a-slug-only-change`     (change new dead)
after    Auditing active changes (3)…                      (reports the real problem)
control  error: duplicate numeric change sequence CHG-0001 (still refused)
```

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-086 | The enumeration test fails against a separate checkout of `origin/main` with the exact production error, `invalid change ID`, and passes here. Its control passes on both binaries, proving the ordinal-free case was excluded from collision detection rather than collision detection being disabled |
