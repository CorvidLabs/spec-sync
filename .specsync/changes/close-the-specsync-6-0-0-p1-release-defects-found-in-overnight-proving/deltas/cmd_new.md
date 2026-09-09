## MODIFIED

### REQUIREMENT REQ-cmd-new-001

`specsync new` SHALL emit every heading in the project's `required_sections` (the same skeleton `add-spec` / `generate` write), not a four-section subset.

Acceptance Criteria
- A spec created by `new` contains Purpose, Public API, Invariants, Behavioral Examples, Error Cases, Dependencies, and Change Log.
- Public API rows are pre-populated from detected exports via `generator::generate_spec`.
- The private `chrono_lite_today` helper is gone; the Change Log date comes from the shared generator.

## ADDED

### REQUIREMENT REQ-cmd-new-004

`specsync new` SHALL refuse a module name under the same rules as `scaffold` (`validate_scaffold_module_name` plus case-collision).

Acceptance Criteria
- Reserved names (`change`, `specs`, `con`), leading dashes, spaces, and path separators exit 1 with `invalid module name` and write nothing.
- A case-fold collision with an existing spec directory is refused before any write.
