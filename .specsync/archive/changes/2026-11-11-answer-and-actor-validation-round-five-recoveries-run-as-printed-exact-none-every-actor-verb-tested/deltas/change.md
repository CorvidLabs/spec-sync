---
change: answer-and-actor-validation-round-five-recoveries-run-as-printed-exact-none-every-actor-verb-tested
artifact: delta
module: change
---

## MODIFIED

### REQUIREMENT REQ-change-150

Interview answers and identity strings SHALL be validated before they are persisted. `change answer` SHALL reject a question key outside the interview set `change answer` accepts, SHALL reject an empty or whitespace-only answer and an answer that parses to no value (`[]` for acceptance criteria; only commas or blanks for the comma-separated lists), and SHALL reject an `affected_specs` entry whose module has no spec at its canonical path (the module's own directory and file spelled exactly, so a variant differing only in ASCII case is refused, and a spec reached through a symlink is refused naming the symlink), unless the project has no spec files and no registry entries; `change new --spec` applies the same rule. A yes/no question (`public_contract`, `architecture_risk`) SHALL take only yes or no, or a spelling the correction ledger reads as one (`y`, `true`, `1`, `n`, `false`, `0`, any case), stored as the choice it names; any other answer SHALL be refused naming the choices and store nothing. An `affected_paths` answer of `[]` SHALL be refused as naming no path. An exact `none` or `[]` SHALL clear `affected_specs` and `dependencies`; `none` SHALL be refused as ambiguous when a change or module is itself named `none`, while `[]` always clears. A module whose spec declares it off its canonical path SHALL be refused naming that spec and saying to move or register it, never suggesting a scaffold that would duplicate it. An owner recovery SHALL NOT print an `affected_specs` answer naming a module with no spec at its canonical path, even in a project with no specs, because verification refuses a declared module without its spec; the hint for a configured owner with no spec says to create its spec first, and approve's ownership refusal and the ship preflight's fallback recovery name each declared module whose spec is missing. A case variant of a module name SHALL be refused naming the module. Actor identity strings SHALL be trimmed and rejected when empty after trimming or when they contain control characters, the line and paragraph separators U+2028 and U+2029, or any format character (general category Cf) or default-ignorable code point as of Unicode 16.0; ZWNJ U+200C and ZWJ U+200D stay allowed. This SHALL hold on approve, reopen, send-back, correct, correct-owner, reconcile-history, retire and the doctor late close through one shared `normalize_actor`, and on review through its stricter reviewer-claim check. A relayed approval's decider SHALL be trimmed the same way as the actor it must match.

Acceptance Criteria
- An unknown question key exits 1, stores nothing in `state.json`, and the error lists the accepted question ids; this covers a hyphen/underscore typo such as `acceptance-criteria`.
- An empty or whitespace-only answer, or one that parses to no value (`[]`, `,`), exits 1 and stores nothing; `none` clears `affected_specs` and `dependencies`.
- An `affected_specs` answer naming a module with no spec at its canonical path exits 1, names that path and `specsync new <module>` or the known modules; an unreadable registry is reported as such.
- An `--actor` value with leading or trailing whitespace is stored trimmed; one that is empty after trimming or holds a refused character exits 1 before any ledger write, for explicit, Git- and environment-derived actors.
- A relayed approval with a padded `--actor` records the same trimmed name as actor and decider, and `change status` reads it.
- An empty `constraints` answer says to answer `none`; an unreadable registry is reported instead of skipping the check; a registered module whose spec file is missing is named as registered.
- Each actor-accepting verb (reopen before moving an archive, retire, correct, correct-owner, reconcile-history, the doctor late close) refuses a bad actor and writes nothing.
- `public_contract maybe` exits 1 naming yes and no and stores nothing; ` N ` is stored as `no`; `affected_paths []` exits 1.
- A declared module whose spec is missing is named in approve's ownership refusal and in the ship preflight's fallback recovery, and no `affected_specs` answer is printed while any listed module lacks its spec.
