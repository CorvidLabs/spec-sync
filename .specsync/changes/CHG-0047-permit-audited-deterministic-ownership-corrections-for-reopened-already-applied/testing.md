---
change: CHG-0047-permit-audited-deterministic-ownership-corrections-for-reopened-already-applied
artifact: testing
---

# Testing

Map `REQ-change-033`, `REQ-cli-args-005`, and `REQ-cmd-change-003` to unit and CLI integration
coverage. The parser regression proves the exact path/spec/actor/reason grammar, while the CLI
integration flow proves deterministic JSON persistence, human next-gate output, and transactional
domain failure.

Also regress `REQ-change-014` so the legacy baseline authority signs its exact protected ledger
path while ordinary archive workspaces remain excluded from delivery manifests.

- Reopen a legacy-style accepted change whose production input is currently owned by an omitted
  canonical module; correct the exact owner, reapprove, verify, reaccept, and assert the signed
  manifest owner without canonical delta replay.
- Assert deterministic text/JSON output and portable definition evidence across different checkout
  roots.
- Reject accepted-but-not-reopened, non-canonical-applied, wrong-state, out-of-scope, unsafe,
  missing, symlinked, pseudo-owner, non-owning-module, duplicate, empty-actor, and empty-reason
  requests without changing any file.
- Tamper sequence, path, module, actor, reason, or vector ordering and require status, strict check,
  verification, acceptance, history reconstruction, and archive preflight to fail closed.
- Prove unrelated changes to affected specs, affected paths, artifacts, answers, dependencies,
  supersedes edges, or deltas remain rejected after correction.
- Prove absent and empty correction fields preserve legacy state JSON and definition digests.
