---
change: answer-and-actor-validation-round-five-recoveries-run-as-printed-exact-none-every-actor-verb-tested
artifact: design
---

# Design

`declare_owners_answer` checks each newly declared owner's canonical spec. Owners without one
are named with `specsync new <module>` before the answer, so every printed command runs as
printed once the earlier ones have. `clearable_list_answer` takes an exact `none` and refuses it
when a change id (dependencies) or a known module (affected_specs) is itself `none`. The answer
`none,` names it instead. `ensure_affected_specs_exist` loads the registry first and reports a
load error. A registered module whose file is missing gets its own message. `normalize_actor`
also refuses U+200B to U+200F, U+2060 to U+2069 and U+FEFF.
