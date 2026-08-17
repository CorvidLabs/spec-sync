## ADDED

### REQUIREMENT REQ-exports-010

An export scan SHALL classify a path that is a directory as its own outcome, never as an unreadable file.

Acceptance Criteria
- Scanning a directory returns a distinct directory outcome, decided before any attempt to read the path as text.
- A directory outcome is never reported as unreadable, so a caller cannot confuse "this is not a file" with "this file could not be read".
- The plain-vector entry points return an empty vector for a directory, so callers that do not inspect the outcome are unaffected.
- The predicate that decides whether a `files:` entry is a directory is shared by every command that asks the question, so a directory cannot be classified one way by one command and differently by another.
