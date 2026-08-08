---
change: CHG-0098-upgrade-site-astro-to-7-2-0-fixing-dependabot-xss-alerts-12-14
artifact: context
---

# Context

## Problem

GitHub Dependabot reported three open Astro XSS advisories against `site/package.json`:

| Alert | Severity | Patched |
|-------|----------|---------|
| #12 Reflected XSS via View Transition animation properties | Moderate | 7.1.0 |
| #13 XSS via unescaped `transition:*` on hydrated islands | Low | 7.0.4 |
| #14 XSS via unescaped spread attribute names in `renderHTMLElement` | Moderate | 7.0.6 |

The site was on `astro@^6.4.6` (resolved 6.4.8). All three require Astro **≥ 7.1.0**.

## Fix

- Bump `astro` to `^7.2.0` (installs 7.2.0)
- Bump `@astrojs/mdx` to `^7.0.5` (Astro 7 peer)
- Add `@astrojs/markdown-remark` + keep the existing remark `.md` link rewrite via `markdown.processor: unified(...)`
- Add `@astrojs/markdown-satteri` for `@astrojs/mdx` peer
- Set `compressHTML: true` to preserve pre-v7 whitespace between inline elements

## Notes

- Docs site only; no product Rust/runtime contract change.
- Site does not use View Transitions / client islands for `transition:*` — still upgrade for dependency hygiene.
