## ADDED

### REQUIREMENT REQ-manifest-002

A Gradle project's module identity SHALL come from its project name.

Acceptance Criteria
- A single-project build is named from a literal `rootProject.name`.
- When `rootProject.name` is unset the project directory name is used, which is Gradle's own default rather than a spec-sync convention.
- A multi-project build continues to use its `include` names.
- No module name is derived from a source path segment, so neither the first nor the last segment of a package hierarchy can become a module.
