---
change: CHG-0108-stop-reporting-success-for-checks-that-did-not-happen-gate-drafts-that-document
artifact: context
---

# Context

## What led here

RC1 stress testing. Two read-only fan-outs — one over adversarial repository shapes,
one over seven real CorvidLabs repos — plus a hand-walked cold-agent path. The three
original RC blockers were confirmed fixed (CHG-0107). These are what the passes found
next, and every one of them was re-verified by hand from a minimal fixture before being
acted on.

They share a shape: **the tool reports success for a check that did not happen**, or
reports a finding it has no evidence for. That is the failure mode a verification tool
cannot ship with, because the signal it emits is indistinguishable from a real one.

## The four

1. **A `draft` spec documenting a nonexistent function passed green.** `status: draft`
   skips section and export validation, and the summary counted the spec as `passed` with
   `100%` coverage and exit 0. `generate` writes new specs as draft, so this is day-one
   state for an adopter — and the current state of `3md` (all three specs) and `attest`
   (its only spec covering 13 Swift files).

2. **A quoted `files:` entry was taken literally** — `- "src/alpha.rs"` reported the file
   missing, then cascaded into a false `Spec documents 'one' but no matching export found
   in source`, because the file was never opened. Frontmatter was still `✓ valid`, so
   nothing pointed at the quotes.

3. **Cold-cache drift noise.** `.specsync/hashes.json` is untracked, so a fresh clone has
   no baseline, and an absent entry classified as changed. A fresh clone of a 33-spec
   project printed 33 `requirements changed` warnings, none real. CI always starts cold.

4. **The coverage-gate remediation reached 8000 characters** on a wide branch — one
   `--path` per file on one line. Introduced by CHG-0107.

## What a session picking this up needs to know

- **`status: draft` means two different things and only one is a problem.** A draft whose
  files do not exist yet is spec-first authoring: the spec is deliberately written before
  the code, nothing could have been validated, and it must keep passing `--strict`. Three
  integration tests pin that contract. A draft whose files *are* present and whose Public
  API names symbols is asserting something checkable and skipping the check.
- **The narrow rule was chosen deliberately over the broad one.** Warning on any draft
  with present source would have required rewriting those three pinned tests. Requiring a
  non-empty Public API as well leaves all three passing **unedited** — the strongest
  available evidence the line is in the right place — while still catching `3md` and
  `attest`, which document real APIs.
- **`--strict` reporting fewer warnings than bare `check` is not a bug.** It looks alarming
  and was reported as one. `--strict` re-validates every spec and so never consults the
  classification that produced the phantom warnings. Making `--strict` propagate them
  would be a regression.

## Ruled out

- **Splitting `passed` from `skipped` in the summary line.** It would break a large number
  of assertions for a cosmetic gain; raising the skip to a warning achieves the actual
  goal, which is that `--strict` gates.
- **Fixing the symlink abort (#546) here.** A single benign intra-project symlink aborts
  `check`, `coverage`, `score`, and `generate` with one line and no validation. The guard
  is a deliberate capability-confinement property with its own test, and resolving targets
  would reintroduce the escape it exists to prevent, TOCTOU included. The likely fix —
  skip and warn, never traverse, always disclose what was excluded from coverage — changes
  coverage denominators and deserves its own change. No CorvidLabs repo hits it today.
- **Unquoting only `files:`.** The same defect hit scalars (`status: "active"`), so the fix
  belongs at the parse layer where it covers every field at once.
