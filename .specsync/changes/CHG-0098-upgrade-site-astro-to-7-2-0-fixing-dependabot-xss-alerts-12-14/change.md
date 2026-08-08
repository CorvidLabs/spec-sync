---
id: CHG-0098-upgrade-site-astro-to-7-2-0-fixing-dependabot-xss-alerts-12-14
state: implementing
type: operations
base_commit: 27c87154b282de36ad4d19302bbf0d3f6726476c
---

# Upgrade site Astro to 7.2.0 fixing Dependabot XSS alerts 12-14

## Intent

Upgrade site Astro to 7.2.0 fixing Dependabot XSS alerts 12-14

## Affected Canonical Specs

- None

## Acceptance Criteria

- site/package.json and lock resolve astro >= 7.1.0 (7.2.0); Dependabot alerts 12, 13, and 14 for Astro XSS are remediated; bun test and bun run build succeed for the site; .md link rewrite still works via unified() processor

## No-spec Rationale

Docs site dependency security patch only; no product/runtime API or canonical module contract change
