---
change: allow-audited-reopening-when-legacy-acceptance-cannot-be-reconstructed
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-094 | The legacy_reopen recovery test fails on the unchanged implementation and passes with the fix through reacceptance and archival. The reconstructible control passes both versions without mutation; the authentication control rejects missing audit fields and tampered closing evidence. |

Use a real Git repository and manifest-less v1 acceptance. Assert archive refusal, matching current digest, anchored verification, then audited reopen and successful reacceptance/archive. Controls cover reconstructible legacy evidence, current modern evidence, and invalid actor/reason or authentication. Run targeted tests before the full verification lane.

The original implementation failed the recovery regression at its current-evidence refusal, while the reconstructible control passed. With the fix, all three legacy_reopen tests pass, including authentication refusal. Format, clippy, and types passed. The full suite passed: 2,458 unit tests and 416 integration tests, using TMPDIR=/var/tmp/specsync751.EvuTH6. The default /tmp has an unrelated .git marker that breaks non-repository fixtures; a first alternate directory containing the word test also trips an existing generator assertion on absolute paths. Neither issue required an implementation change.
