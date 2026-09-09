## ADDED

### REQUIREMENT REQ-config-014

A JSON config file that exists but fails to parse SHALL set `load_error` with the same wording as an unreadable file, so `load_config` refuses instead of applying built-in defaults.

Acceptance Criteria
- Malformed `specsync.json` and `.specsync/config.json` both set `load_error` containing `could not be loaded`.
- `rules`, `rehash`, `compact`, `deps`, `archive-tasks`, and `view` exit 1 over that config and do not rewrite files.
- Callers that opt into `load_config_allowing_unloadable` still see the standing-in defaults plus the recorded error.
