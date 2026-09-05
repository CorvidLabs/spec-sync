# Lesson bundle — allow-audited-reopening-when-legacy-acceptance-cannot-be-reconstructed

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Allow audited reopening when legacy acceptance cannot be reconstructed
- **Kind**: Feature
- **Specs**: change
- **Paths**: src/change.rs, src/change_tests.rs, specs/change/
- **Acceptance**: Legacy accepted records without a manifest can reopen with explicit actor and reason when historical reconstruction fails despite current inputs and anchored evidence; the audit preserves prior evidence and records the cause; reconstructible legacy and current modern evidence still refuse; fresh verification and acceptance permit archival.

## Evidence

- Verification commit: `c483d350e3df09216af1e9d2508ddac5bc1f8227`
- Base commit: `6b1717038edb467d95bb483861f0c076da76deb5`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

<!-- What led here: the problem, and how it was noticed. -->

<!-- What a session picking this up mid-flight needs to know: constraints,
     prior attempts, anything already ruled out. -->

Issue #751 reports legacy accepted packages whose current raw input digest matches but whose acceptance-transition trees cannot reproduce that digest. Archive refuses reconstruction while reopen reports current evidence. Work is isolated on fix/751-legacy-reopen; the existing Trust-pin change is outside this scope.

## From the change's design.md

# Design

Add a distinct ReopenCauseV1 variant for unreconstructible manifest-less workflow-v1 acceptance. Evaluate reconstruction after existing evidence authentication, and do not let exact/successor input checks veto that recovery cause. Preserve all prior evidence and require fresh verification and closing. Modern records do not use this legacy path.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-094 | The legacy_reopen recovery test fails on the unchanged implementation and passes with the fix through reacceptance and archival. The reconstructible control passes both versions without mutation; the authentication control rejects missing audit fields and tampered closing evidence. |

Use a real Git repository and manifest-less v1 acceptance. Assert archive refusal, matching current digest, anchored verification, then audited reopen and successful reacceptance/archive. Controls cover reconstructible legacy evidence, current modern evidence, and invalid actor/reason or authentication. Run targeted tests before the full verification lane.

The original implementation failed the recovery regression at its current-evidence refusal, while the reconstructible control passed. With the fix, all three legacy_reopen tests pass, including authentication refusal. Format, clippy, and types passed. The full suite passed: 2,458 unit tests and 416 integration tests, using TMPDIR=/var/tmp/specsync751.EvuTH6. The default /tmp has an unrelated .git marker that breaks non-repository fixtures; a first alternate directory containing the word test also trips an existing generator assertion on absolute paths. Neither issue required an implementation change.

## Where these lessons go

- `specs/change/context.md`
