## ADDED

### REQUIREMENT REQ-config-017

`specs_dir`, every `source_dirs` entry and `schema_dir` SHALL be relative paths with no `..` component. Any other value SHALL make the config unloadable, with a message that names the field and the value, for TOML and JSON configs alike. The MCP server SHALL refuse such a config for the one request, as an error that names the field, and keep serving.

Acceptance Criteria
- `specs_dir` as `../outside/specs`, an absolute path or `specs/../../outside/specs`, a `source_dirs` entry `../outside` and a `schema_dir` of `../outside` each make `deps` fail naming the field, and nothing outside is listed.
- The MCP server answers `specsync_generate` with a `specs_dir` of `../outside/specs`, and `specsync_coverage` with a `source_dirs` entry of `../outside/src`, with a tool error naming the field, and writes nothing outside.
