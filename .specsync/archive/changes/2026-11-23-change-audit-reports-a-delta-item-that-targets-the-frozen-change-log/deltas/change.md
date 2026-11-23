## ADDED

### REQUIREMENT REQ-change-170

`change audit` SHALL report, for each approved, implementing or verifying change, the refusal REQ-change-159 gives a delta item that targets the frozen `## Change Log` section, with the message `change check` gives, prefixed by the change id, and SHALL ask it in the order `change check` asks it: after the definition approval and the delta files hold, and before the approved delta bodies and bases. An archived change SHALL NOT be judged again.

Acceptance Criteria
- An approved, implementing or verifying change whose approved delta holds a `SPEC SECTION Change Log` item, as an older SpecSync approved it, makes `change audit` report the refusal `change check` gives, prefixed by the change id, and `change audit --strict` exits 1.
- An archived change whose package holds such an item is not reported.
- An active change whose delta holds no such item audits as before, and so does a project with no active change.
