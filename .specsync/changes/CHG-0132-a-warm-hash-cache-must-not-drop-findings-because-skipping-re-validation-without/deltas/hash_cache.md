## ADDED

### REQUIREMENT REQ-hash-cache-004

A cached spec's validation result SHALL be stored and replayed.

Acceptance Criteria
- The result recorded when a spec was last validated survives in the cache alongside its hash.
- A spec skipped as unchanged replays that result rather than contributing nothing.
- Editing a spec or any file it maps re-validates it and overwrites the stored result.
- The cache continues to skip re-validation, not merely re-extraction; the optimisation is preserved and the verdict is not lost.
