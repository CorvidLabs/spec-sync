---
change: audit-every-module-context-md-against-the-source-and-correct-every-false-or-stale-claim
artifact: testing
---

# Testing

**No test is added, and none is possible.** This change edits prose in 34 `context.md` companion
files and touches no source, no behaviour, and no canonical spec text. There is nothing on either
side of the edit for an assertion to discriminate between.

The discrimination protocol requires that a new assertion be shown to FAIL against a binary built
from a separate checkout of unfixed `main`. Any test written for this change would either:

- assert on the prose itself, which passes on unfixed `main` for one file and fails for another
  purely because the string differs — a change-detector, not a discriminator; or
- assert on the code the prose describes, which passes identically on both sides, because the code
  is what was already correct. The prose was the thing that was wrong.

Saying that plainly is the honest outcome. Adding a test that cannot fail for the right reason
would misrepresent this change as behaviour-verified.

## What was verified instead

The evidence for this change is the reproduced measurement, not a test. Every count and every
symbol claim is recorded in `research.md` with the command that produced it, so a reviewer can
re-run each one against this tree and get the number now written in the file. That is the whole
verification surface, and it is deliberately re-runnable rather than pinned.

## Gates that must still pass

- `cargo fmt --check` — clean (no source changed).
- `cargo clippy -- -D warnings` — clean (no source changed).
- `cargo test` — 2,407 unit and 407 integration tests, unaffected by a prose-only change; run to
  prove exactly that.
- `specsync change check` — targeted verification for this change.
- `specsync change audit --strict` — exit 0.

## One characterization worth recording

`tests/integration/comment.rs::comment_suppresses_configured_command_output_but_check_streams_it`
still passes, but no longer for the reason its name gives: since #543, `comment` runs no configured
verification command at all, so the absence of that output is trivial rather than suppressed. The
test is not touched here — it is outside this change's scope — but `specs/cmd_comment/context.md`
now says what it actually proves, so the next reader does not mistake it for evidence of a quiet
execution path that no longer exists.
