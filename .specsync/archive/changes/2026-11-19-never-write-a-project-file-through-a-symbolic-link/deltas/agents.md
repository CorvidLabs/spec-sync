## ADDED

### REQUIREMENT REQ-agents-011

`agents install` and `agents uninstall` SHALL inspect, create, rewrite and remove generated agent artifacts and `.specsync/agent-artifacts.json` only through the project file functions of `util` (REQ-util-002). A target that is a symbolic link to a regular file inside the project SHALL be written through. A target, or a directory on the way to it, that leaves the project through a symbolic link, a dangling link, and a target that is not a regular file SHALL be refused with an error that names the path, leaving the link and what it points at unchanged and creating nothing outside the project. `agents install` SHALL refuse before it writes any artifact.

Acceptance Criteria
- A dangling link at `.claude/skills/spec-sync/SKILL.md`: `agents install --claude` fails naming that path, creates nothing at the link's target, and writes no other artifact.
- `.specsync/agent-artifacts.json` linked to a file outside the project: `agents install` fails naming it and the outside file is unchanged.
- `.claude` linked to a directory outside the project: nothing is created there.
- With no link, `agents install` and `agents uninstall` behave as before.
