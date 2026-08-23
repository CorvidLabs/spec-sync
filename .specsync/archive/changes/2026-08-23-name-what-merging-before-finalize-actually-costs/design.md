---
change: name-what-merging-before-finalize-actually-costs
artifact: design
---

# Design

## Extract, then correct

The two ship-status warnings were inline string literals inside a large function, which is why the
wording had no test. They become `merge_before_finalize_warning(still_active: bool)`, a pure
function beside the existing `multi_active_ordering_warnings` — the same shape, for the same
reason.

That makes the disclosure pinnable. The test asserts the phrases that carry the second-order cost:
whose work is blocked, what is blocked, and both exits.

## Four sites, one message

- `src/commands/change.rs` — the two `ship-status` warnings, now via the shared function
- `src/commands/change.rs` — the `ship_next` hint, corrected inline
- `src/cli.rs` — the `finalize` help text

The CLI help matters as much as the runtime warning: it is where someone reads about the verb
before they are in a position to need the warning.

## Deliberately not done

Naming the specific predecessors at risk. The tool knows the change's delivery inputs and the
other records' states, so it could enumerate them — but that is a behaviour change with its own
cost model, and the wording captures most of the value. Recorded on #687 rather than smuggled in.
