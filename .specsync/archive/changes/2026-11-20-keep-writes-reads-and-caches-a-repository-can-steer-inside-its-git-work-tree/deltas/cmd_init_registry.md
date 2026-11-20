## ADDED

### REQUIREMENT REQ-cmd-init-registry-004

`init-registry` SHALL create `.specsync/registry.toml` with the exclusive create of the confined project write (REQ-util-002), so a `.specsync` that leads out of the repository is refused and receives nothing, and a registry that appears meanwhile is kept.

Acceptance Criteria
- A `.specsync` linked to an outside directory gets no `registry.toml`, and the command fails.
