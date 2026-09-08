---
change: repair-executable-lifecycle-demos
artifact: design
---

# Design

Read the generated ID from `change new --json`. Populate the actual selected artifacts with meaningful fixture intent, and remove verification-command fallbacks because `change check` does not execute them. Use explicit fixture actor labels, current check/commit, immediate review/finalize, and a separate archive commit for each change before verifying the next. Preserve the dependent-before-prerequisite refusal control. Run the five-epic crate tests explicitly in an isolated Cargo target directory and report success only after the executable tests pass. Update the three example READMEs to describe prerequisites, temporary artifacts, fixture identities, and structural versus product verification. Add `.github/scripts/test-sdd-examples.py` to execute the demos using a supplied built binary with bounded subprocess deadlines and inspect completion/archive evidence. Invoke it in the existing Ubuntu CI test job after cargo build. Negative controls must prove retired IDs fail and a failing product test prevents the five-epic success report. No canonical module contract or library implementation changes.
