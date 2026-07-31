## MODIFIED

### REQUIREMENT REQ-ignore-001

Ignore rules SHALL combine project and inline suppressions deterministically, expose suppressed
warning counts, and limit each rule to its documented matching scope.

Acceptance Criteria

- Category names are case-insensitive, treat `_` and `-` equivalently, and retain documented aliases.
- `.specsyncignore` supports global and path-prefix rules; inline directives support comma-separated
  categories and require a closing marker.
- Only classified warnings are suppressible; errors and unknown warning text remain visible.
- Check/report structured output includes deterministic `suppressed_warnings` details and text output
  states when warnings were hidden.
- Strict exit behavior uses unsuppressed findings only.
