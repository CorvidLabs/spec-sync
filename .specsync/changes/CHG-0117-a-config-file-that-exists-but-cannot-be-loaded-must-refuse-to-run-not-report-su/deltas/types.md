## ADDED

### REQUIREMENT REQ-types-007

Configuration SHALL record when it is the built-in defaults standing in for a config file
that exists but could not be loaded.

Acceptance Criteria
- A successful load records no such condition.
- A config file that cannot be read, and one that cannot be parsed, both record it, naming the file.
- The absence of a config file is not recorded as a failure, because the defaults are then the intended configuration.
