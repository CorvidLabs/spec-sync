## ADDED

### REQUIREMENT REQ-cmd-wizard-003

The wizard SHALL create the spec file only through `create_project_file` of `util` (REQ-util-002) and its companions through the generator (REQ-generator-010), so a spec path that is a dangling link, or leaves the project through a link, is refused with an error naming the path and nothing is created outside the project.

Acceptance Criteria
- The wizard's write step, driven with a dangling link at the spec path, returns an error naming the path and creates nothing at the link's target.
- The same step with the spec directory linked to a directory outside the project creates nothing there.
