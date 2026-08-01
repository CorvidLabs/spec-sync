## MODIFIED

### REQUIREMENT REQ-cmd-change-006

The change command adapter SHALL render draft next-action guidance that prefers completing incomplete selected artifacts over recommending definition approval, using lightweight artifact completeness without digest-bearing loaders for text mode.

Acceptance Criteria

- Text and JSON next-action guidance do not recommend change approve for interview-complete drafts with incomplete selected artifacts.
- Completeness guidance remains available without writing digests into cleartext text sinks.
