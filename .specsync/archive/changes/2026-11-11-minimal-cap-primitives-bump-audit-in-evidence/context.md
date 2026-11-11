---
change: minimal-cap-primitives-bump-audit-in-evidence
artifact: context
---

# Context

corvid-agent+t370 on #1054: cargo update re-resolved 9 crates windows-sys edges to 0.52.0 (they are not yanked; any cargo touch re-normalizes them), so the minimal change is a hand edit of the two cap-primitives lines, verified with cargo build --locked. cargo audit output is captured as evidence. tasks.md ticks only what has run.
