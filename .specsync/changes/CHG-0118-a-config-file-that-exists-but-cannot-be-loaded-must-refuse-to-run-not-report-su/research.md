---
change: CHG-0118-a-config-file-that-exists-but-cannot-be-loaded-must-refuse-to-run-not-report-su
artifact: research
---

# Research

## How it was found

An agent probing the commands with **no drill coverage at all** — `score`, `deps`, `comment`,
`watch`, `report`, `merge`, `import`, `rules`, `compact` — rather than from a bug report.
It built two trees differing by one character and compared them.

Every earlier defect in this series was found by looking at a *command*. This one is at the
configuration layer beneath all of them, which is why probing commands nobody had tested
surfaced it and eleven rounds of fixing individual commands had not.

## Verified independently before acting

Reproduced by hand with a control, because two findings in an earlier agent pass were
inverted:

```
valid config    → exit 1, ✗ Missing required section: ## Threat Model
one ] deleted   → exit 0, ✓ All required sections present
```

## Escape hatches, all ineffective

| attempt | result |
|---|---|
| `--strict` | exit 0 |
| `--force` | exit 0 |
| `--json` | `"passed": true`, `"errors": []` |
| `score` | **rose** from 96/100 to 100/100 |
| `score --min-score 90` | exit 0 |

The score movement is the clearest evidence of the mechanism: the project lost the very rule
it was failing, so its grade improved.

## Precision worth recording

The run reaches exit 0 only because the spec independently satisfies the built-in defaults.
The defect is that **configured-beyond-default rules vanish without failing the run**.

This repository's own `.specsync/config.toml` happens to set `required_sections` equal to the
seven defaults, so that field is coincidentally harmless here — `[rules]` thresholds and any
added section are not. A project whose config matches the defaults would never notice; one
that configured anything real is silently returned to the behaviour it rejected.

## Scope discovered while implementing

Four fallback sites, in two distinct shapes. The parse-failure site in `validator.rs` — the
one the repro actually goes through — does not share the textual form of the two
unreadable-file sites in `config.rs`. Enumerating by behaviour rather than by pattern is what
kept the fix from being a no-op against its own issue.

## Related

- #560 — same shape at a different layer: a branch whose comment stated `--strict` must gate,
  which it did not.
- #553 — `✓ All required sections present` printed when frontmatter could not be parsed. The
  same sentence, one layer up.
