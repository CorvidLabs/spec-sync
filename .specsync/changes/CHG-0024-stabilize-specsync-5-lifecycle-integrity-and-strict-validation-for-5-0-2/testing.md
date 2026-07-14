---
change: CHG-0024-stabilize-specsync-5-lifecycle-integrity-and-strict-validation-for-5-0-2
artifact: testing
---

# Testing

| Requirement | Focused Evidence |
|-------------|------------------|
| `REQ-change-022` | Unit and integration fixtures create duplicate active/archive sequences verify exact diagnostics preserve the historical baseline and simulate divergent branch claims. |
| `REQ-change-023` | Direct `specsync check` and indirect Fledge-style re-entry fail once; native commands execute once; a failed attempt is retained and a corrected retry passes. |
| `REQ-change-024` | Frontmatter expansion makes an accepted predecessor stale; exact implementing and passed-verifying successors govern it; failed draft no-spec and partial successors remain red; accepted successor leaves strict clean. |
| `REQ-change-025` | A registry mapping `client-api = specs/client/client-api.spec.md` receives both spec and requirements updates while hostile mappings fail closed. |
| `REQ-validator-002` | Mapped and unmapped HTML fixtures report `1/1` and `0/1`; excluded assets stay absent; static content has no export error. |
| `REQ-validator-003` | Every generated marker fails strict with path and line; concrete prose passes; similar prose and fenced examples pass. |

Release evidence uses `fledge lanes run repo`, `fledge spec check --strict`, focused Cargo tests, `fledge trust verify`, Augur, Attest, and all hosted Linux macOS Windows action and CodeQL checks.
