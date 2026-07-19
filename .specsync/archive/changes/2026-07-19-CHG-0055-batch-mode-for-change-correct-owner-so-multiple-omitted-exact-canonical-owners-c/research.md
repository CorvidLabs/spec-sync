---
change: CHG-0055-batch-mode-for-change-correct-owner-so-multiple-omitted-exact-canonical-owners-c
artifact: research
---

# Research

## Observed rollout cost

Trust 1.1.1 adoption required 11–19 sequential `correct-owner` cycles in consumer repos
(swift-asa-viewer, py-algochat, swift-algotest). Each cycle re-ran the full verify lane because
only one owner could be appended per definition approval.

## Existing contract

REQ-change-033 already requires append-only sequenced corrections, scope checks, canonical
frontmatter ownership, and transactional rejection for invalid single requests. Batch mode must
reuse that validator rather than invent a second ownership policy.

## Alternatives considered

| Option | Rejected because |
|--------|------------------|
| One combined audit entry for the whole batch | Breaks contiguous sequence / portable reconstruction |
| Apply valid entries and report invalid ones | Silent partial apply violates fail-closed lifecycle |
| Only `--all-missing` | Still need explicit path lists when modules differ per path |
