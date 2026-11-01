## ADDED

### REQUIREMENT REQ-hooks-005

The system SHALL write hook instruction blocks, the Git pre-commit hook and Claude Code hook settings without U+2013 (en dash) or U+2014 (em dash), and SHALL keep recognizing the begin sentinel and pre-commit hook body that releases up to and including 6.1.1 wrote.

Acceptance Criteria
- `hooks install` for every target writes no U+2013 or U+2014.
- A new managed block begins with ``<!-- specsync:hook:begin (managed by `specsync hooks`; do not edit inside) -->`` and ends with the unchanged `<!-- specsync:hook:end -->`.
- `hooks uninstall` removes a block framed by the 6.1.1 begin sentinel exactly as it removes one framed by the new sentinel, preserving content before and after it.
- `hooks uninstall --precommit` recognizes a legacy unmarked appended block and a legacy whole-file hook whose body is either the 6.1.1 text or the current text, and removes project-keyed managed blocks whatever body they hold.
- The `Spec-Sync Integration` and `Spec-Sync Rules` detection markers, the end sentinel and the project-keyed pre-commit markers are unchanged.
- Installation does not rewrite a block that is already present.
