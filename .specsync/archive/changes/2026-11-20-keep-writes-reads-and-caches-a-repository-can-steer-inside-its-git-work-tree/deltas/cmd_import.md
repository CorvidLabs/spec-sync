## ADDED

### REQUIREMENT REQ-cmd-import-004

`import --from-dir` SHALL read each entry within the spec bound: by the project rule when the directory, as the person wrote it, lies in the project, and within that directory itself when it lies outside. An entry refused there SHALL be reported as a failed read, with the reason, and SHALL never reach a spec draft.

Acceptance Criteria
- An entry linked outside the repository in `docs/` is refused, and its text is in no spec; the other entries are imported.
- In an outside directory, a link inside it is imported and a link out of it is refused.
