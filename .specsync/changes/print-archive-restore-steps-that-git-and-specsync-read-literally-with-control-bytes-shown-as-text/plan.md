---
change: print-archive-restore-steps-that-git-and-specsync-read-literally-with-control-bytes-shown-as-text
artifact: plan
---

# Plan

## Approach

Make the printed restore step and its probes literal for Git and SpecSync, and print control
bytes as escapes, in `corrupt_archive_recovery`, `corrupt_archive_signal` and
`archive_package_differs_from`.

## Out of scope

Next actions outside the corrupt-archive restore step that interpolate an id (a follow-up issue).
Validating archive directory names at load (it would make existing archives unreadable).

## Steps

1. Carry over CHANGE-7.b (Leif, 2026-11-06) and the red tests from the g-literal branch; commit red.
2. Literal pathspecs in the probes and the printed commands.
3. `--` before the id in the printed status commands.
4. `shell_word` for control bytes.
5. Targeted tests (literal guidance, handoff, corrupt-archive recovery), fmt, clippy, then the
   Chough 0.6.0 session and the full local suite before review.

## Risks

Existing tests that pin the old exact strings need their expected text updated. `$'…'` needs
bash, zsh, ksh or a POSIX.1-2024 sh, and it is printed only for names that hold a control byte.

## Constraints consulted

#901: every interpolated word is quoted as one shell word; the step is built from literals, the
id and the package path only, never a digest. CHANGE-7.a.2: the printed step is the exact restore
command.
