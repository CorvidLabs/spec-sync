---
change: CHG-0126-the-config-refusal-must-guard-both-loaders-because-load-config-is-a-second-door
artifact: design
---

# Design

Invert the default. `load_config` refuses; `load_config_allowing_unloadable` is
the raw loader, and callers that need it must ask by name.

The alternative — adding the guard to each direct caller — is what produced this
bug. #570 threaded it to one entry point and the second was missed, silently,
because nothing fails when a caller forgets. Naming the permissive variant makes
the omission visible at the call site and makes the safe path the one you get by
default.

Only the repair paths opt out: `wizard` and the registry initialiser, whose job
is to fix a broken config and which would otherwise be unable to run on the
project that needs them most.

This does not make the guard compiler-enforced — a new caller could still write
`load_config_allowing_unloadable` without cause. But it inverts which mistake is
silent: forgetting the guard is now impossible, and bypassing it deliberately is
a word you have to type.
