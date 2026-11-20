## ADDED

### REQUIREMENT REQ-config-018

Source auto-detection SHALL skip a child of the project root that is a symbolic link and SHALL never follow a link at the root of its walk.

Acceptance Criteria
- A committed `lib -> ../outside/lib` is not detected as a source folder.
