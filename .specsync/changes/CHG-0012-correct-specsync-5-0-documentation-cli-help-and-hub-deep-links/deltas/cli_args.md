## ADDED

### REQUIREMENT REQ-cli-args-003
The shared CLI grammar SHALL describe the files produced by initialization and full spec scaffolding accurately.

Acceptance Criteria
- `init` help names `.specsync/config.toml` rather than the retired root JSON configuration.
- Global option help names the canonical configuration and every accepted output format.
- `add-spec` help describes the required companion set and optional design artifact.
- `new --full` help lists the required companion files and identifies `design.md` as optional.
- Help-only corrections do not change argument parsing or command behavior.
