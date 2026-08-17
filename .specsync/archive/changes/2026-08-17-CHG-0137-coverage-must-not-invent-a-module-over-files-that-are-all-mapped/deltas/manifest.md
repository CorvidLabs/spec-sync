## ADDED

### REQUIREMENT REQ-manifest-018

A manifest module SHALL carry the source paths it declares, so that a consumer can judge the module against its own files rather than against its name alone.

Acceptance Criteria
- Every discovered manifest module exposes the source paths attributed to it by the manifest it came from.
- A module whose manifest declares no source paths exposes an empty set rather than being omitted, so a consumer can tell "declares nothing" from "was not discovered".
- Cargo, Swift and Gradle discovery all populate the field, including the Gradle single-project fallback that derives its name from the root directory.
