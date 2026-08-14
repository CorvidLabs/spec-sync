## ADDED

### REQUIREMENT REQ-config-009

Loading SHALL distinguish an absent config file from one that exists and could not be
loaded.

Acceptance Criteria
- An absent file yields the built-in defaults with no recorded failure.
- An unreadable file yields the built-in defaults with the failure recorded.
- A file that fails to parse yields the built-in defaults with the failure recorded.
