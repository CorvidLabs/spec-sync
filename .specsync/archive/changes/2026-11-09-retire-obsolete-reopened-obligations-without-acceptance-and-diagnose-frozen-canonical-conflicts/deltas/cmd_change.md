## ADDED

### REQUIREMENT REQ-cmd-change-recovery-retirement-001

The change command SHALL expose audited retirement as explicit nonacceptance and report its authority claim, replacement and remaining historical limitations without describing it as accepted.

Acceptance Criteria
- The operation requires actor, reason, replacement and authority source; repeated identical disposition is safe; retired-only dependencies and incoming semantic claims remain refused; custody is checked before relying on retirement.


- Report source-kind and per-input classification, preserve historical receipt formats, and refuse a corrupt/missing source rather than infer a weaker source.
