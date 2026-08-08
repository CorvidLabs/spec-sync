---
change: CHG-0098-upgrade-site-astro-to-7-2-0-fixing-dependabot-xss-alerts-12-14
artifact: testing
---

# Testing

```bash
cd site
bun test
bun run build
# installed version must be >= 7.1.0
node -e "console.log(require('./node_modules/astro/package.json').version)"
```

Acceptance:

- All site unit tests pass
- Static build completes (docs/blog/examples routes)
- Resolved `astro` version is 7.2.0 (or later 7.x that still covers 7.1.0+)
