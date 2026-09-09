## ADDED

### REQUIREMENT REQ-config-015

A TOML config file that exists but cannot be used SHALL set `load_error` with the same wording as a JSON parse-fail, so `load_config` refuses instead of applying built-in defaults through the silent line scanner.

Acceptance Criteria
- Malformed TOML, an empty file, a directory standing in for the config path, and an unknown `enforcement` enum value all set `load_error` containing `could not be loaded`.
- `rules`, `rehash`, `compact`, `deps`, `archive-tasks`, and `view` exit 1 over those configs and do not rewrite files; the same verbs still succeed when no config file exists.
- `load_toml_config` parses through `parse_config_content_checked` (real `toml::from_str` plus `validate_toml_config_types`) rather than calling `parse_toml_config` directly.
- Callers that opt into `load_config_allowing_unloadable` still see the standing-in defaults plus the recorded error.
