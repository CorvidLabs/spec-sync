## ADDED

### REQUIREMENT REQ-generator-007

Generation SHALL preserve a scoped or nested manifest module identity while naming its spec from the module path leaf and expanding companion references to that actual filename. Retained CLI output write failures SHALL return an error.

Acceptance Criteria
- @scope/demo generates specs/@scope/demo/demo.spec.md with a valid quoted full module identity.
- Text, JSON, batch and ambient generation use the same destination layout.
- Companion references name demo.spec.md and existing files remain unchanged on rerun.
- Retained directory creation or spec publication failures cannot return successful empty output.
