---
change: name-a-held-cargo-build-directory-lock-before-verification-blocks-on-it-and-reap-the-verification-child-s-process-group
artifact: requirements
---

# Requirements

Adds `REQ-change-091` and extends contract item 5 of the `change` module. The normative wording is
in `deltas/change.md`; this artifact records what the requirement has to be *shaped* like, and why.

- **The claim must be read from the lock, never inferred.** `change check` on this repository takes
  four to five minutes, so any elapsed-time signal fires on the healthy case. The notice exists
  only when a non-blocking exclusive acquisition reports contention.
- **Silence is a permitted answer and must stay one.** A build directory that cannot be derived
  exactly from argv and environment produces no notice. Naming a lock the command will never wait
  on restores the ambiguity the notice removes, so it is worse than saying nothing.
- **Say only what the platform can support.** A holder PID is printed where the operating system
  reports lock ownership. Where it does not, the notice names the command that answers the question
  instead of guessing — the same claim, at the precision available.
- **Reaping is an obligation on the child's shape, not on the signal path.** The requirement is
  that the child leads its own process group and that the group is ended on an unwind or a
  catchable terminating signal. A `SIGKILL`ed parent is explicitly out of scope, in the requirement
  text, because pretending otherwise would make the requirement unverifiable.
