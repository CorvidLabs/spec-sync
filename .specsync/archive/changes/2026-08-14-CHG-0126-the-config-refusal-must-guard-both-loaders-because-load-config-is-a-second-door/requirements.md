---
change: CHG-0126-the-config-refusal-must-guard-both-loaders-because-load-config-is-a-second-door
artifact: requirements
---

# Requirements

`REQ-config-00N` — the default configuration loader SHALL refuse when a config
file exists and cannot be used, and a permissive loader SHALL be available under
a name that says so.

`REQ-cmd-wizard-00N`, `REQ-cmd-init-registry-00N` — the repair paths SHALL
request the permissive loader explicitly and keep working over a broken config.

Out of scope: the hand-rolled TOML scanner's inability to detect a parse error,
which is a separate defect on a separate path.
