---
change: CHG-0118-a-config-file-that-exists-but-cannot-be-loaded-must-refuse-to-run-not-report-su
artifact: context
---

# Context

## What led here

CorvidLabs/spec-sync#570, found by an agent probing commands that have no drill coverage,
then verified by hand against a valid-config control.

Two trees, byte-identical except one missing `]` in `.specsync/config.toml`. The config sets
`required_sections` to the seven built-ins plus `"Threat Model"`; the spec has no Threat
Model section.

```
$ specsync check --root ok        # valid config
  ✗ Missing required section: ## Threat Model
exit 1

$ specsync check --root broken    # one bracket deleted, stdout as CI captures it
  ✓ All required sections present
1 specs checked: 1 passed, 0 warning(s), 0 failed
exit 0
```

The warning existed — on **stderr only**. A CI job capturing stdout saw a clean pass.

## Why this is the worst instance of the class

Ten prior fixes share one shape: a category empty for want of *input*, read as want of
*problems*. Every one of them disabled a single check. This is that bug at the
**configuration layer**, so it disables every rule the project configured — `required_sections`,
`[rules]` thresholds, `exclude_patterns` — simultaneously, from one typo.

It survived every escape hatch: `--strict`, `--force`, `--json` (payload `"passed": true`
with empty `errors`), and `score --min-score 90` (which rose from 96 to a perfect 100 once
the rule it was failing had been thrown away).

`✓ All required sections present` is a claim about a section list that no longer exists.

## The intent was already written down

The fallback sites carry comments saying a present-but-unreadable config "must fail loud
rather than silently downgrade enforcement to defaults". They then warn and return the
defaults. Same shape as #560: the requirement was understood, recorded, and not delivered.

## Why refusing is right

A project writes a config file precisely because the built-in defaults are not enough.
Substituting the defaults for a typo hands back the behaviour the project explicitly
rejected, and calls it success. There is no reading of a malformed config under which
"pretend it says nothing" is the safe interpretation.
