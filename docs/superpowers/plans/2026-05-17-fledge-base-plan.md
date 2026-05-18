# Marketing site rebuild — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the mdBook-only GitHub Pages site at `corvidlabs.github.io/fledge` with an Astro + MDX marketing site that has Merlin parity (hero, terminal demo, six-pillars, blog, docs) plus a first-class searchable plugin registry with per-plugin pages.

**Architecture:** Single Astro 5 project under `site/`, Bun-driven, dark warm-rust theme. mdBook content migrates into an Astro `docs` content collection. The plugin registry is baked into the build by a prebuild script that queries the GitHub API for `org:CorvidLabs+fledge-plugin-*` (plus community allowlist), writes a slim `plugins.json` index and per-plugin `plugins/{slug}.json` files containing the rendered README. Deployed via GitHub Pages with a weekly cron so the registry refreshes without a code change.

**Tech Stack:** Astro 5, `@astrojs/mdx`, `@astrojs/sitemap`, Bun runtime + test runner, TypeScript, `marked` + `isomorphic-dompurify` for README rendering. No CSS framework — hand-rolled CSS using custom properties.

**Spec:** `docs/superpowers/specs/2026-05-17-marketing-site-design.md`

**Working branch:** `docs/marketing-site-spec` (existing; planning + implementation continue here)

---

## File Structure

The plan creates one new top-level directory `site/`, leaves the rest of the repo untouched until Phase 9, then deletes `docs/` (the mdBook tree) and `.github/workflows/docs.yml` in the cutover.

```
site/
  astro.config.mjs                    # Astro config (site, base, integrations)
  package.json
  bun.lockb
  tsconfig.json
  bunfig.toml                         # test config
  README.md                           # how to dev / build the site
  public/
    favicon.svg
    og-default.png
    robots.txt
    plugins/{old-mdbook-path}.html    # generated 1-step redirects (Phase 9)
  scripts/
    build-plugin-registry.ts          # prebuild: GH → site/src/data/
    build-plugin-registry.test.ts
    plugin-helpers.ts                 # pure helpers (testable)
    plugin-helpers.test.ts
    render-readme.ts                  # markdown → sanitized HTML
    render-readme.test.ts
    related-plugins.ts                # topic-overlap ranking
    related-plugins.test.ts
    community-allowlist.json          # additional plugin authors
    generate-doc-redirects.ts         # writes site/public/<old-path>.html
  src/
    env.d.ts
    components/
      Header.astro
      Footer.astro
      Button.astro
      Badge.astro
      Terminal.astro
      Pillar.astro
      PluginCard.astro
      ExampleCard.astro
      PostCard.astro
      CategoryTag.astro
      Callout.astro
      TableOfContents.astro
      Sidebar.astro
      Pager.astro
    layouts/
      BaseLayout.astro
      ArticleLayout.astro
      DocsLayout.astro
    content/
      config.ts
      docs/                            # migrated from docs/src/
      examples/                        # MDX walkthroughs (3 seed posts)
      blog/                            # MDX posts (2 seed posts)
    data/
      plugins.json                     # generated; gitignored
      plugins/                         # generated; gitignored
        {slug}.json
    pages/
      index.astro
      404.astro
      plugins/
        index.astro
        [slug].astro
      examples/
        index.astro
        [...slug].astro
      docs/
        index.astro
        [...slug].astro
      blog/
        index.astro
        [...slug].astro
    styles/
      globals.css                      # CSS custom properties + reset + base
.github/workflows/pages.yml            # new; replaces docs.yml
```

Files in `site/src/data/` are committed in skeleton form (so the dev server runs without a network) but rebuilt on every CI deploy.

---

## Phase 1 — Scaffold

**Goal of phase:** `cd site && bun run dev` opens an empty-but-styled Astro site.

### Task 1.1: Create the Astro project structure

**Files:**
- Create: `site/package.json`
- Create: `site/tsconfig.json`
- Create: `site/astro.config.mjs`
- Create: `site/bunfig.toml`
- Create: `site/.gitignore`
- Create: `site/README.md`
- Create: `site/src/env.d.ts`
- Modify: `.gitignore` (add `site/dist`, `site/node_modules`, `site/.astro`)

- [ ] **Step 1: Add the project metadata**

Create `site/package.json`:

```json
{
  "name": "fledge-site",
  "type": "module",
  "version": "0.0.1",
  "private": true,
  "scripts": {
    "dev": "astro dev",
    "prebuild": "bun scripts/build-plugin-registry.ts && bun scripts/generate-doc-redirects.ts",
    "build": "astro build",
    "preview": "astro preview",
    "test": "bun test",
    "fmt": "prettier --write src scripts",
    "lint": "astro check"
  },
  "dependencies": {
    "@astrojs/mdx": "^4.0.0",
    "@astrojs/sitemap": "^3.0.0",
    "astro": "^5.0.0",
    "isomorphic-dompurify": "^2.18.0",
    "marked": "^15.0.0"
  },
  "devDependencies": {
    "@types/bun": "latest",
    "@types/node": "^20.0.0",
    "prettier": "^3.3.0",
    "prettier-plugin-astro": "^0.14.0",
    "typescript": "^5.6.0"
  }
}
```

Create `site/tsconfig.json`:

```json
{
  "extends": "astro/tsconfigs/strict",
  "include": ["src", "scripts", "**/*.astro"],
  "compilerOptions": {
    "jsx": "preserve",
    "types": ["astro/client", "bun-types"]
  }
}
```

Create `site/astro.config.mjs`:

```js
import { defineConfig } from 'astro/config'
import mdx from '@astrojs/mdx'
import sitemap from '@astrojs/sitemap'

export default defineConfig({
  site: 'https://corvidlabs.github.io',
  base: '/fledge/',
  trailingSlash: 'never',
  integrations: [mdx(), sitemap()],
})
```

Create `site/bunfig.toml`:

```toml
[test]
preload = []
```

Create `site/.gitignore`:

```
node_modules
dist
.astro
src/data/plugins.json
src/data/plugins/
public/og-cards/
```

Create `site/README.md`:

```markdown
# fledge — marketing site

Astro + MDX site. Hosted on GitHub Pages at https://corvidlabs.github.io/fledge.

## Dev

    bun install
    bun run dev        # localhost:4321/fledge/

## Build

    bun run build      # writes site/dist/

## Test

    bun test
```

Create `site/src/env.d.ts`:

```ts
/// <reference path="../.astro/types.d.ts" />
/// <reference types="astro/client" />
```

- [ ] **Step 2: Update repo-level gitignore**

In `.gitignore`, append (after the `.superpowers/` line):

```
site/node_modules
site/dist
site/.astro
```

- [ ] **Step 3: Install dependencies**

Run from repo root:

```bash
cd site && bun install
```

Expected: bun.lockb appears, exit 0.

- [ ] **Step 4: Commit**

```bash
git add site/package.json site/tsconfig.json site/astro.config.mjs \
        site/bunfig.toml site/.gitignore site/README.md \
        site/src/env.d.ts site/bun.lockb .gitignore
git commit -m "feat(site): scaffold Astro project structure"
```

---

### Task 1.2: Add base CSS and a placeholder home page

**Files:**
- Create: `site/src/styles/globals.css`
- Create: `site/src/layouts/BaseLayout.astro`
- Create: `site/src/pages/index.astro`

- [ ] **Step 1: Add the CSS custom properties (the rust palette)**

Create `site/src/styles/globals.css`:

```css
:root {
  --bg: #0c0a09;
  --bg-raised: #18120e;
  --bg-raised-2: #1c1917;
  --border: #2a201a;
  --border-strong: #3a2c22;
  --text: #f5f5f4;
  --text-muted: #c2b9b1;
  --text-dim: #9c948d;
  --accent: #ea580c;
  --accent-bright: #fdba74;
  --accent-deep: #9a3412;
  --accent-muted: rgba(234, 88, 12, 0.10);
  --accent-glow: rgba(234, 88, 12, 0.15);
  --focus: #fdba74;
  --radius: 10px;
  --font: -apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif;
  --serif: "Iowan Old Style", "Apple Garamond", "Baskerville", Georgia, serif;
  --mono: "JetBrains Mono", "SF Mono", Menlo, Consolas, monospace;
  --tap: 44px;
  --max-w: 1180px;
  --container-pad: 32px;
}

* { box-sizing: border-box; margin: 0; padding: 0; }
html { font-size: 100%; -webkit-text-size-adjust: 100%; }
body {
  background: var(--bg); color: var(--text);
  font-family: var(--font); font-size: 1rem; line-height: 1.6;
  -webkit-font-smoothing: antialiased;
}
ul, ol { list-style: none; }
a { color: inherit; text-decoration: none; }
img, svg { display: block; max-width: 100%; }
button { font-family: inherit; cursor: pointer; }

.container { max-width: var(--max-w); margin: 0 auto; padding: 0 var(--container-pad); }

a:focus-visible, button:focus-visible, input:focus-visible,
select:focus-visible, [tabindex]:focus-visible {
  outline: 2px solid var(--focus);
  outline-offset: 3px;
  border-radius: 4px;
}

.skip-link {
  position: absolute; top: -100px; left: 8px;
  background: var(--accent); color: #1a0f08;
  padding: 12px 18px; border-radius: 8px;
  font-weight: 600; text-decoration: none; z-index: 100;
  transition: top .15s;
}
.skip-link:focus { top: 8px; }

.sr-only {
  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0,0,0,0); white-space: nowrap; border: 0;
}
```

- [ ] **Step 2: Add the BaseLayout**

Create `site/src/layouts/BaseLayout.astro`:

```astro
---
import '../styles/globals.css'
interface Props {
  title: string
  description?: string
}
const { title, description = "Dev lifecycle CLI. Get your projects ready to fly." } = Astro.props
const base = import.meta.env.BASE_URL
---
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>{title}</title>
  <meta name="description" content={description} />
  <link rel="icon" type="image/svg+xml" href={`${base}favicon.svg`} />
  <meta property="og:title" content={title} />
  <meta property="og:description" content={description} />
</head>
<body>
  <a class="skip-link" href="#main">Skip to main content</a>
  <slot />
</body>
</html>
```

- [ ] **Step 3: Add a minimal home page**

Create `site/src/pages/index.astro`:

```astro
---
import BaseLayout from '../layouts/BaseLayout.astro'
---
<BaseLayout title="fledge — dev lifecycle CLI">
  <main id="main" class="container" style="padding-top: 80px;">
    <h1>fledge</h1>
    <p>Marketing site under construction. <a href="/fledge/docs">Docs →</a></p>
  </main>
</BaseLayout>
```

- [ ] **Step 4: Verify dev server runs**

```bash
cd site && bun run dev
```

Expected: server starts on `http://localhost:4321/fledge/`. Visit it — see the placeholder. Stop the server (Ctrl+C).

- [ ] **Step 5: Commit**

```bash
git add site/src/styles/globals.css site/src/layouts/BaseLayout.astro \
        site/src/pages/index.astro
git commit -m "feat(site): add base layout, globals.css, and placeholder home"
```

---

### Task 1.3: Add the favicon and OG default image

**Files:**
- Create: `site/public/favicon.svg`
- Create: `site/public/robots.txt`
- Create: `site/public/og-default.png` (placeholder — solid color)

- [ ] **Step 1: Add the favicon as an inline SVG**

Create `site/public/favicon.svg`:

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
  <defs>
    <linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#fdba74"/>
      <stop offset="60%" stop-color="#ea580c"/>
      <stop offset="100%" stop-color="#9a3412"/>
    </linearGradient>
  </defs>
  <rect width="64" height="64" rx="14" fill="url(#g)"/>
  <text x="32" y="44" text-anchor="middle" font-family="-apple-system, system-ui, sans-serif"
        font-weight="800" font-size="36" fill="#1a0f08">f</text>
</svg>
```

- [ ] **Step 2: Add robots.txt**

Create `site/public/robots.txt`:

```
User-agent: *
Allow: /

Sitemap: https://corvidlabs.github.io/fledge/sitemap-index.xml
```

- [ ] **Step 3: Add a placeholder OG image**

Use `convert` (ImageMagick) if available, otherwise drop in a 1200×630 PNG by hand. Acceptable placeholder: a solid `#0c0a09` rectangle. If ImageMagick is installed:

```bash
cd site && convert -size 1200x630 xc:'#0c0a09' \
  -fill '#fdba74' -font Helvetica -pointsize 96 -gravity center \
  -annotate +0+0 'fledge' public/og-default.png
```

If not available, create `site/public/og-default.png` as a 1×1 transparent PNG (acceptable for v1; replace later):

```bash
printf '\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15\xc4\x89\x00\x00\x00\rIDATx\x9cc\x00\x01\x00\x00\x05\x00\x01\r\n-\xb4\x00\x00\x00\x00IEND\xaeB`\x82' > public/og-default.png
```

- [ ] **Step 4: Commit**

```bash
git add site/public/favicon.svg site/public/robots.txt site/public/og-default.png
git commit -m "feat(site): add favicon, robots.txt, OG placeholder"
```

---

## Phase 2 — Design system primitives

**Goal of phase:** all reusable components exist with their final styling; a 404 page composes them as a smoke test.

### Task 2.1: Button component

**Files:**
- Create: `site/src/components/Button.astro`

- [ ] **Step 1: Implement the component**

Create `site/src/components/Button.astro`:

```astro
---
interface Props {
  href?: string
  variant?: 'primary' | 'secondary' | 'ghost'
  type?: 'button' | 'submit'
  class?: string
  ariaLabel?: string
}
const { href, variant = 'primary', type = 'button', class: extraClass = '', ariaLabel } = Astro.props
const cls = `btn btn-${variant} ${extraClass}`.trim()
const Tag = href ? 'a' : 'button'
---
<Tag class={cls} {...href ? { href } : { type }} aria-label={ariaLabel}>
  <slot />
</Tag>

<style is:global>
  .btn {
    display: inline-flex; align-items: center; gap: 8px;
    padding: 12px 20px; border-radius: 8px;
    font-size: 0.9375rem; font-weight: 500; text-decoration: none;
    border: 1px solid transparent; cursor: pointer;
    min-height: 44px; font-family: inherit;
    transition: all .15s ease;
  }
  .btn-primary { background: var(--accent); color: #1a0f08; font-weight: 600; }
  .btn-primary:hover { background: var(--accent-bright); }
  .btn-secondary { background: transparent; color: var(--text); border-color: var(--border-strong); }
  .btn-secondary:hover { border-color: var(--accent); color: var(--accent-bright); background: var(--accent-muted); }
  .btn-ghost { background: transparent; color: var(--text-muted); padding: 12px 14px; }
  .btn-ghost:hover { color: var(--text); }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add site/src/components/Button.astro
git commit -m "feat(site): add Button component"
```

---

### Task 2.2: Badge component

**Files:**
- Create: `site/src/components/Badge.astro`

- [ ] **Step 1: Implement**

Create `site/src/components/Badge.astro`:

```astro
---
interface Props { dot?: boolean }
const { dot = false } = Astro.props
---
<span class="badge">
  {dot && <span class="dot" aria-hidden="true" />}
  <slot />
</span>

<style is:global>
  .badge {
    display: inline-flex; align-items: center; gap: 8px;
    padding: 6px 14px; border-radius: 999px;
    background: var(--accent-muted); color: var(--accent-bright);
    font-size: 0.875rem; font-weight: 500;
    border: 1px solid rgba(234,88,12,0.25);
  }
  .badge .dot { width: 8px; height: 8px; border-radius: 50%;
    background: var(--accent-bright); box-shadow: 0 0 8px var(--accent-bright); }
  @media (prefers-reduced-motion: no-preference) {
    .badge .dot { animation: badge-pulse 2s infinite ease-in-out; }
    @keyframes badge-pulse { 0%,100% { opacity: 1; } 50% { opacity: .55; } }
  }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add site/src/components/Badge.astro
git commit -m "feat(site): add Badge component"
```

---

### Task 2.3: Terminal component

**Files:**
- Create: `site/src/components/Terminal.astro`

- [ ] **Step 1: Implement**

Create `site/src/components/Terminal.astro`:

```astro
---
interface Props {
  title?: string
  ariaLabel?: string
}
const { title = 'terminal', ariaLabel = 'Terminal example' } = Astro.props
---
<figure class="terminal" aria-label={ariaLabel}>
  <div class="term-header">
    <div class="term-dots" aria-hidden="true">
      <span class="d1"></span><span class="d2"></span><span class="d3"></span>
    </div>
    <span class="term-title">{title}</span>
  </div>
  <div class="term-body">
    <slot />
  </div>
</figure>

<style is:global>
  .terminal {
    background: var(--bg-raised); border: 1px solid var(--border-strong);
    border-radius: var(--radius); overflow: hidden;
    font-family: var(--mono); font-size: 0.9375rem; line-height: 1.85;
    box-shadow: 0 24px 60px rgba(0,0,0,0.45), 0 0 60px var(--accent-glow);
  }
  .term-header { display: flex; align-items: center; gap: 12px; padding: 12px 16px;
    background: rgba(255,255,255,.03); border-bottom: 1px solid var(--border); }
  .term-dots { display: flex; gap: 6px; }
  .term-dots span { width: 12px; height: 12px; border-radius: 50%; }
  .term-dots .d1 { background: #ff5f57; }
  .term-dots .d2 { background: #febc2e; }
  .term-dots .d3 { background: #28c840; }
  .term-title { color: var(--text-muted); font-size: 0.8125rem; }
  .term-body { padding: 20px 22px; }
  .term-line { white-space: pre-wrap; word-break: break-word; }
  .term-prompt { color: var(--accent-bright); }
  .term-out { color: var(--text-muted); }
  .term-good { color: var(--accent-bright); }
  .term-comment { color: var(--text-dim); font-style: italic; }
  .blink { display: inline-block; width: 8px; height: 16px;
    background: var(--accent-bright); vertical-align: middle; margin-left: 2px; }
  @media (prefers-reduced-motion: no-preference) {
    .blink { animation: blink 1.1s infinite; }
    @keyframes blink { 0%,49% { opacity: 1; } 50%,100% { opacity: 0; } }
  }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add site/src/components/Terminal.astro
git commit -m "feat(site): add Terminal component"
```

---

### Task 2.4: Header component

**Files:**
- Create: `site/src/components/Header.astro`

- [ ] **Step 1: Implement**

Create `site/src/components/Header.astro`:

```astro
---
import Button from './Button.astro'
const base = import.meta.env.BASE_URL
const current = Astro.url.pathname.replace(/\/$/, '') || base.replace(/\/$/, '')
const VERSION = 'v1.4.1'
const links = [
  { href: `${base}plugins`, label: 'Plugins' },
  { href: `${base}examples`, label: 'Examples' },
  { href: `${base}docs`, label: 'Docs' },
  { href: `${base}blog`, label: 'Blog' },
]
const isActive = (href: string) => current === href.replace(/\/$/, '')
---
<header class="site-header" role="banner">
  <div class="nav-inner">
    <a href={base} class="nav-logo" aria-label="fledge home">
      <span class="glyph" aria-hidden="true">f</span>
      <span>fledge</span>
      <span class="nav-version">{VERSION}</span>
    </a>
    <nav aria-label="Primary">
      <ul class="nav-links">
        {links.map(l => (
          <li>
            <a href={l.href} class={isActive(l.href) ? 'active' : ''}
               aria-current={isActive(l.href) ? 'page' : undefined}>{l.label}</a>
          </li>
        ))}
      </ul>
    </nav>
    <div class="nav-cta">
      <a href="https://github.com/CorvidLabs/fledge" class="btn btn-ghost"
         aria-label="View on GitHub (opens in new tab)" target="_blank" rel="noopener">
        GitHub <span aria-hidden="true">↗</span>
      </a>
      <Button href={`${base}docs/getting-started/installation`} variant="primary">Install</Button>
    </div>
  </div>
</header>

<style is:global>
  .site-header { border-bottom: 1px solid var(--border); position: sticky; top: 0;
    background: rgba(12,10,9,.85); backdrop-filter: blur(14px); z-index: 10; }
  .nav-inner { display: flex; align-items: center; justify-content: space-between;
    padding: 16px 32px; max-width: 1180px; margin: 0 auto; gap: 16px; }
  .nav-logo { display: flex; align-items: center; gap: 10px;
    font-weight: 600; font-size: 1.0625rem; text-decoration: none;
    color: var(--text); letter-spacing: -0.01em; min-height: 44px; }
  .nav-logo .glyph { width: 28px; height: 28px; border-radius: 7px;
    background: linear-gradient(135deg, var(--accent-bright), var(--accent) 60%, var(--accent-deep));
    display: grid; place-items: center; color: #1a0f08; font-weight: 800;
    font-size: 0.875rem; box-shadow: 0 0 20px var(--accent-glow); }
  .nav-version { color: var(--text-dim); font-size: 0.875rem; font-family: var(--mono); }
  .nav-links { display: flex; gap: 4px; }
  .nav-links a { color: var(--text-muted); text-decoration: none;
    font-size: 0.9375rem; padding: 10px 14px; border-radius: 6px;
    min-height: 44px; display: inline-flex; align-items: center; }
  .nav-links a:hover { color: var(--accent-bright); background: rgba(255,255,255,0.03); }
  .nav-links a.active { color: var(--accent-bright); background: var(--accent-muted); }
  .nav-cta { display: flex; gap: 8px; align-items: center; }
  @media (max-width: 760px) { .nav-links { display: none; } }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add site/src/components/Header.astro
git commit -m "feat(site): add Header component"
```

---

### Task 2.5: Footer component

**Files:**
- Create: `site/src/components/Footer.astro`

- [ ] **Step 1: Implement**

Create `site/src/components/Footer.astro`:

```astro
---
const base = import.meta.env.BASE_URL
---
<footer role="contentinfo">
  <div class="container">
    <div class="foot-grid">
      <div class="foot-brand">
        <p class="logo"><span aria-hidden="true">◆</span> fledge</p>
        <p class="tag">Dev lifecycle CLI. Get your projects ready to fly.</p>
        <p class="tag">A <a href="https://corvidlabs.xyz">CorvidLabs</a> project.</p>
      </div>
      <div class="foot-col">
        <h2>Product</h2>
        <a href={`${base}plugins`}>Plugins</a>
        <a href={`${base}examples`}>Examples</a>
        <a href={`${base}docs/changelog`}>Changelog</a>
      </div>
      <div class="foot-col">
        <h2>Resources</h2>
        <a href={`${base}docs`}>Documentation</a>
        <a href="https://github.com/CorvidLabs/fledge/blob/main/AGENTS.md">AGENTS.md</a>
        <a href={`${base}blog`}>Blog</a>
      </div>
      <div class="foot-col">
        <h2>Community</h2>
        <a href="https://github.com/CorvidLabs/fledge">GitHub</a>
        <a href="https://crates.io/crates/fledge">Crates.io</a>
        <a href="https://github.com/CorvidLabs/homebrew-tap">Homebrew</a>
      </div>
    </div>
    <div class="foot-bottom">
      <p>© 2026 CorvidLabs · MIT licensed</p>
      <p>Built with fledge <span aria-hidden="true">◆</span></p>
    </div>
  </div>
</footer>

<style is:global>
  footer { border-top: 1px solid var(--border); padding: 60px 0 30px;
    color: var(--text-muted); font-size: 0.9375rem; }
  .foot-grid { display: grid; grid-template-columns: 1.5fr repeat(3, 1fr);
    gap: 40px; margin-bottom: 36px; }
  .foot-brand { display: flex; flex-direction: column; gap: 12px; }
  .foot-brand .logo { display: flex; align-items: center; gap: 8px;
    color: var(--text); font-weight: 600; font-size: 1.0625rem; }
  .foot-brand .tag { color: var(--text-muted); font-size: 0.9375rem; line-height: 1.5; }
  .foot-brand .tag a { color: var(--accent-bright); }
  .foot-brand .tag a:hover { text-decoration: underline; }
  .foot-col h2 { color: var(--text); font-size: 0.9375rem; margin-bottom: 14px; font-weight: 600; }
  .foot-col a { display: block; color: var(--text-muted); font-size: 0.9375rem; padding: 6px 0; }
  .foot-col a:hover { color: var(--accent-bright); }
  .foot-bottom { border-top: 1px solid var(--border); padding-top: 24px;
    display: flex; justify-content: space-between; color: var(--text-dim);
    font-size: 0.875rem; flex-wrap: wrap; gap: 12px; }
  @media (max-width: 760px) { .foot-grid { grid-template-columns: 1fr; } }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add site/src/components/Footer.astro
git commit -m "feat(site): add Footer component"
```

---

### Task 2.6: Wire Header + Footer into BaseLayout

**Files:**
- Modify: `site/src/layouts/BaseLayout.astro`

- [ ] **Step 1: Replace the body content with Header + slot + Footer**

Replace the `<body>` block in `site/src/layouts/BaseLayout.astro` with:

```astro
<body>
  <a class="skip-link" href="#main">Skip to main content</a>
  <Header />
  <slot />
  <Footer />
</body>
```

And add imports at the top of the frontmatter (after `import '../styles/globals.css'`):

```ts
import Header from '../components/Header.astro'
import Footer from '../components/Footer.astro'
```

- [ ] **Step 2: Verify dev server still renders**

```bash
cd site && bun run dev
```

Visit `http://localhost:4321/fledge/` — Header and Footer should appear around the placeholder copy. Stop server.

- [ ] **Step 3: Commit**

```bash
git add site/src/layouts/BaseLayout.astro
git commit -m "feat(site): wire Header and Footer into BaseLayout"
```

---

### Task 2.7: Callout, CategoryTag, Pillar components

**Files:**
- Create: `site/src/components/Callout.astro`
- Create: `site/src/components/CategoryTag.astro`
- Create: `site/src/components/Pillar.astro`

- [ ] **Step 1: Callout**

Create `site/src/components/Callout.astro`:

```astro
---
interface Props { type?: 'note' | 'warn' | 'tip' }
const { type = 'note' } = Astro.props
---
<aside class={`callout callout-${type}`} role="note">
  <slot />
</aside>

<style is:global>
  .callout { padding: 16px 18px; border-left: 3px solid var(--accent);
    background: var(--accent-muted); border-radius: 0 8px 8px 0; margin: 20px 0; }
  .callout p:last-child { margin-bottom: 0; }
  .callout strong { color: var(--accent-bright); }
  .callout-warn { border-left-color: #f87171; background: rgba(248,113,113,0.08); }
  .callout-warn strong { color: #fca5a5; }
  .callout-tip { border-left-color: #34d399; background: rgba(52,211,153,0.08); }
  .callout-tip strong { color: #6ee7b7; }
</style>
```

- [ ] **Step 2: CategoryTag**

Create `site/src/components/CategoryTag.astro`:

```astro
---
type Category = 'announce' | 'plugin' | 'release' | 'workflow' | 'tutorial'
interface Props { category: Category }
const labels: Record<Category, string> = {
  announce: 'Announcement',
  plugin: 'Plugin',
  release: 'Release',
  workflow: 'Workflow',
  tutorial: 'Tutorial',
}
const { category } = Astro.props
---
<span class={`tag tag-${category}`}>{labels[category]}</span>

<style is:global>
  .tag {
    display: inline-flex; align-items: center; gap: 6px;
    padding: 3px 10px; border-radius: 4px; font-size: 0.75rem; font-weight: 600;
    letter-spacing: 0.06em; text-transform: uppercase; border: 1px solid transparent;
  }
  .tag-announce  { background: rgba(248,113,113,0.10); color: #f87171; border-color: rgba(248,113,113,0.3); }
  .tag-plugin    { background: rgba(52,211,153,0.10);  color: #34d399; border-color: rgba(52,211,153,0.3); }
  .tag-release   { background: rgba(253,186,116,0.12); color: #fdba74; border-color: rgba(253,186,116,0.3); }
  .tag-workflow  { background: rgba(96,165,250,0.10);  color: #60a5fa; border-color: rgba(96,165,250,0.3); }
  .tag-tutorial  { background: rgba(192,132,252,0.10); color: #c084fc; border-color: rgba(192,132,252,0.3); }
</style>
```

- [ ] **Step 3: Pillar**

Create `site/src/components/Pillar.astro`:

```astro
---
interface Props { numeral: string; title: string; command: string }
const { numeral, title, command } = Astro.props
---
<li class="pillar">
  <p class="num">{numeral}</p>
  <h3>{title}</h3>
  <p><slot /></p>
  <code>{command}</code>
</li>

<style is:global>
  .pillar { padding: 28px 24px; background: var(--bg); transition: background .15s; }
  .pillar:hover { background: var(--bg-raised); }
  .pillar .num { font-family: var(--serif); font-size: 1rem;
    color: var(--accent-bright); margin-bottom: 14px; font-style: italic; }
  .pillar h3 { font-size: 1.25rem; margin-bottom: 8px; letter-spacing: -0.005em; }
  .pillar p { color: var(--text-muted); font-size: 0.9375rem;
    line-height: 1.6; margin-bottom: 14px; }
  .pillar code { font-family: var(--mono); font-size: 0.875rem;
    color: var(--accent-bright); display: block; padding-top: 10px;
    border-top: 1px dashed var(--border-strong); }
</style>
```

- [ ] **Step 4: Commit**

```bash
git add site/src/components/Callout.astro site/src/components/CategoryTag.astro \
        site/src/components/Pillar.astro
git commit -m "feat(site): add Callout, CategoryTag, Pillar components"
```

---

### Task 2.8: 404 page (smoke-tests the design system)

**Files:**
- Create: `site/src/pages/404.astro`

- [ ] **Step 1: Implement**

Create `site/src/pages/404.astro`:

```astro
---
import BaseLayout from '../layouts/BaseLayout.astro'
import Button from '../components/Button.astro'
const base = import.meta.env.BASE_URL
---
<BaseLayout title="404 — fledge">
  <main id="main" class="container" style="padding: 120px 0 80px; text-align: center;">
    <p style="color: var(--accent-bright); font-family: var(--serif); font-style: italic; font-size: 1.25rem; margin-bottom: 8px;">404</p>
    <h1 style="font-size: 2.4rem; letter-spacing: -0.02em; margin-bottom: 12px;">This page took flight without us.</h1>
    <p style="color: var(--text-muted); font-size: 1.05rem; margin: 0 auto 28px; max-width: 480px;">
      The page you're looking for doesn't exist (any more). Try the docs or the plugin registry.
    </p>
    <div style="display: flex; gap: 12px; justify-content: center; flex-wrap: wrap;">
      <Button href={`${base}docs`} variant="primary">Read the docs</Button>
      <Button href={`${base}plugins`} variant="secondary">Browse plugins</Button>
    </div>
  </main>
</BaseLayout>
```

- [ ] **Step 2: Verify it builds**

```bash
cd site && bun run build
```

Expected: `site/dist/404.html` exists, exit 0.

- [ ] **Step 3: Commit**

```bash
git add site/src/pages/404.astro
git commit -m "feat(site): add 404 page"
```

**🛑 Checkpoint:** Phase 2 done. Header, footer, design tokens, and primitives all in place. Visual sanity-check by running `bun run dev` and visiting `/` (basic layout) and a bogus URL like `/nope` (404).

---

## Phase 3 — Home page

**Goal of phase:** the `/` route matches the home-v3 mockup.

### Task 3.1: Home — hero + stats sections

**Files:**
- Modify: `site/src/pages/index.astro`

- [ ] **Step 1: Replace the home page**

Overwrite `site/src/pages/index.astro`:

```astro
---
import BaseLayout from '../layouts/BaseLayout.astro'
import Badge from '../components/Badge.astro'
import Button from '../components/Button.astro'
import Terminal from '../components/Terminal.astro'
const base = import.meta.env.BASE_URL
---
<BaseLayout title="fledge — get your projects ready to fly">
<main id="main">

<section class="hero">
  <div class="container">
    <div class="hero-grid">
      <div>
        <Badge dot>31 plugins shipping in v1.4</Badge>
        <h1 class="hero-h1">Get your projects<br>ready to <span class="fly">fly.</span></h1>
        <p class="hero-sub">One CLI for the dev loop. Any language. JSON by default. Scaffold, run, ship — without the bash spaghetti.</p>
        <div class="hero-actions">
          <Button href={`${base}docs/getting-started`} variant="primary">Get started <span aria-hidden="true">→</span></Button>
          <Button href={`${base}plugins`} variant="secondary">Browse 31 plugins</Button>
        </div>
        <p class="hero-fineprint">Install in a single command: <code>cargo install fledge</code></p>
      </div>
      <div>
        <Terminal title="~/my-cli — fledge">
          <div class="term-line term-comment"># nothing → shipped, in three commands</div>
          <div class="term-line"><span class="term-prompt">$ </span>fledge templates init my-cli -t rust-cli</div>
          <div class="term-line term-out">  ✓ Scaffolded my-cli/</div>
          <div class="term-line"><span class="term-prompt">$ </span>fledge lanes init</div>
          <div class="term-line term-out">  ✓ fledge.toml: build, test, lint, ci</div>
          <div class="term-line"><span class="term-prompt">$ </span>fledge lanes run ci</div>
          <div class="term-line term-good">  ✓ build (1.8s)</div>
          <div class="term-line term-good">  ✓ test (24 passed, 0.6s)</div>
          <div class="term-line term-good">  ✓ lint</div>
          <div class="term-line term-good">  ★ ci passed in 12s</div>
          <div class="term-line"><span class="term-prompt">$ </span><span class="blink" aria-hidden="true" /></div>
        </Terminal>
      </div>
    </div>
  </div>
</section>

<section class="stats" aria-label="Key numbers">
  <div class="container">
    <ul class="stats-grid">
      <li class="stat"><div class="num">31</div><div class="lbl">plugins shipping</div><div class="sub">official + community</div></li>
      <li class="stat"><div class="num">6</div><div class="lbl">pillars</div><div class="sub">scaffold · run · spec · AI · ship · extend</div></li>
      <li class="stat"><div class="num">∞</div><div class="lbl">languages</div><div class="sub">Rust, TS, Python, Go, anything</div></li>
      <li class="stat"><div class="num">1</div><div class="lbl">binary</div><div class="sub">install, done</div></li>
    </ul>
  </div>
</section>

</main>
</BaseLayout>

<style>
  main { background-image: radial-gradient(ellipse 1200px 600px at 50% -200px, rgba(234,88,12,0.12), transparent 70%); }
  .hero { padding: 80px 0 60px; }
  .hero-grid { display: grid; grid-template-columns: 1.05fr 1fr; gap: 56px; align-items: center; }
  .hero-h1 { font-size: clamp(2.4rem, 5vw, 3.6rem); line-height: 1.05; letter-spacing: -0.025em; margin: 22px 0; }
  .hero-h1 .fly { background: linear-gradient(120deg, var(--accent-bright) 0%, var(--accent) 60%, var(--accent-deep) 100%);
    -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;
    font-style: italic; font-family: var(--serif); font-weight: 500; }
  .hero-sub { color: var(--text-muted); font-size: 1.125rem; max-width: 480px; margin-bottom: 28px; line-height: 1.6; }
  .hero-actions { display: flex; gap: 12px; margin-bottom: 24px; flex-wrap: wrap; }
  .hero-fineprint { color: var(--text-muted); font-size: 0.9375rem; }
  .hero-fineprint code { background: var(--bg-raised); padding: 4px 8px; border-radius: 4px;
    border: 1px solid var(--border); color: var(--accent-bright);
    font-family: var(--mono); font-size: 0.875rem; }
  .stats { padding: 60px 0; border-top: 1px solid var(--border); border-bottom: 1px solid var(--border);
    margin-top: 40px; background: linear-gradient(180deg, transparent, rgba(234,88,12,0.025), transparent); }
  .stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 32px; }
  .stat .num { font-size: 2.5rem; font-weight: 700; letter-spacing: -0.02em;
    color: var(--accent-bright); font-family: var(--serif); line-height: 1; }
  .stat .lbl { color: var(--text); font-size: 1rem; margin-top: 8px; font-weight: 500; }
  .stat .sub { color: var(--text-muted); font-size: 0.875rem; margin-top: 4px; }
  @media (max-width: 900px) { .hero-grid, .stats-grid { grid-template-columns: 1fr; } }
</style>
```

- [ ] **Step 2: Verify in dev server**

```bash
cd site && bun run dev
```

Visit `http://localhost:4321/fledge/`. Hero + terminal + stats should render. Stop server.

- [ ] **Step 3: Commit**

```bash
git add site/src/pages/index.astro
git commit -m "feat(home): hero, terminal demo, and stats row"
```

---

### Task 3.2: Home — pillars grid

**Files:**
- Modify: `site/src/pages/index.astro`

- [ ] **Step 1: Add the Pillar import and section**

In `site/src/pages/index.astro` frontmatter, after `import Terminal`:

```ts
import Pillar from '../components/Pillar.astro'
```

Insert this `<section>` after the closing `</section>` of `.stats`:

```astro
<section class="pillars" aria-labelledby="pillars-h">
  <div class="container">
    <header class="section-head">
      <p class="section-eyebrow">Six pillars</p>
      <h2 id="pillars-h">Everything you need from <em>nothing</em><br>to <em>shipped</em>, in one binary.</h2>
      <p>Each pillar is a focused subcommand. Plugins extend any of them. Pick what you need.</p>
    </header>
    <ul class="pillars-grid">
      <Pillar numeral="i."  title="Scaffold" command="$ fledge templates init">Built-in templates for Rust, TS, Python, Go. Tera placeholders. Community registry.</Pillar>
      <Pillar numeral="ii." title="Run"      command="$ fledge lanes run ci">Task runner with composable lanes. Parallel/sequential. File watcher built in.</Pillar>
      <Pillar numeral="iii." title="Spec"    command="$ fledge spec check">Specs as constraints. Validate code matches spec. Agent-friendly source of truth.</Pillar>
      <Pillar numeral="iv."  title="AI"      command="$ fledge review">Spec-aware ask and review. Works with Claude, Ollama, OpenAI, any provider.</Pillar>
      <Pillar numeral="v."   title="Ship"    command="$ fledge work commit --ai">Branch → commit (AI optional) → push → release → changelog.</Pillar>
      <Pillar numeral="vi."  title="Extend"  command="$ fledge plugins install">Plugin protocol in any language. 31 plugins to install. Or write your own.</Pillar>
    </ul>
  </div>
</section>
```

Append to the `<style>` block:

```css
.pillars { padding: 100px 0 40px; }
.section-head { margin-bottom: 48px; }
.section-eyebrow { color: var(--accent-bright); font-size: 0.875rem; font-weight: 600;
  letter-spacing: 0.12em; text-transform: uppercase; margin-bottom: 14px; }
.section-head h2 { font-size: clamp(1.85rem, 3.5vw, 2.6rem); letter-spacing: -0.02em;
  max-width: 580px; line-height: 1.15; margin-bottom: 14px; }
.section-head h2 em { font-style: italic; font-family: var(--serif);
  color: var(--accent-bright); font-weight: 400; }
.section-head p { color: var(--text-muted); font-size: 1.0625rem; max-width: 580px; }
.pillars-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1px;
  background: var(--border); border: 1px solid var(--border);
  border-radius: var(--radius); overflow: hidden; }
@media (max-width: 900px) { .pillars-grid { grid-template-columns: 1fr; } }
```

- [ ] **Step 2: Verify**

`bun run dev`, check the pillars grid renders as a 2×3 (3-column at desktop). Stop.

- [ ] **Step 3: Commit**

```bash
git add site/src/pages/index.astro
git commit -m "feat(home): six pillars grid"
```

---

### Task 3.3: Home — examples teaser + CTA banner

**Files:**
- Modify: `site/src/pages/index.astro`

- [ ] **Step 1: Add the sections**

Append these sections inside `<main>`, after the pillars `</section>`:

```astro
<section class="examples-teaser" aria-labelledby="ex-h">
  <div class="container">
    <header class="section-head">
      <p class="section-eyebrow">Walkthroughs</p>
      <h2 id="ex-h">See it on a <em>real</em> project.</h2>
      <p>End-to-end examples — every command, every file, every output.</p>
    </header>
    <ul class="ex-grid">
      <li><a href={`${base}examples/rust-cli`} class="ex-card">
        <span class="ex-tag">Rust CLI</span>
        <h3>Build a Rust CLI end-to-end</h3>
        <p>From templates init through release bump, with conventional commits.</p>
        <div class="ex-meta">8 steps · 12 min</div>
      </a></li>
      <li><a href={`${base}examples/ts-bun`} class="ex-card">
        <span class="ex-tag">TS + Bun</span>
        <h3>TypeScript project with Bun</h3>
        <p>Language detection, lane composition, ship workflow.</p>
        <div class="ex-meta">6 steps · 8 min</div>
      </a></li>
      <li><a href={`${base}examples/custom-plugin`} class="ex-card">
        <span class="ex-tag">Plugins</span>
        <h3>Wire up a custom plugin</h3>
        <p>Build a fledge-plugin-* with the v1 protocol.</p>
        <div class="ex-meta">10 steps · 20 min</div>
      </a></li>
    </ul>
  </div>
</section>

<section class="cta-banner" aria-labelledby="cta-h">
  <div class="container">
    <h2 id="cta-h">Stop wrangling shell scripts.<br>Take <em>flight.</em></h2>
    <p>Install fledge and run your first lane in under a minute.</p>
    <div class="cta-install">
      <span aria-hidden="true" style="color: var(--accent-bright);">$</span>
      <code>cargo install fledge</code>
    </div>
    <div class="cta-actions">
      <Button href={`${base}docs`} variant="primary">Read the docs</Button>
      <Button href={`${base}plugins`} variant="secondary">Browse plugins</Button>
    </div>
  </div>
</section>
```

Append to `<style>`:

```css
.examples-teaser { padding: 80px 0; background: linear-gradient(180deg, transparent, rgba(234,88,12,0.03)); }
.ex-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; margin-top: 32px; }
.ex-card { display: block; padding: 26px; background: var(--bg-raised);
  border: 1px solid var(--border); border-radius: var(--radius);
  transition: all .15s; }
.ex-card:hover { border-color: var(--accent); transform: translateY(-2px); }
.ex-tag { display: inline-block; padding: 4px 10px; border-radius: 4px;
  background: var(--accent-muted); color: var(--accent-bright);
  font-size: 0.8125rem; font-weight: 600;
  margin-bottom: 14px; letter-spacing: 0.05em; text-transform: uppercase; }
.ex-card h3 { font-size: 1.125rem; margin-bottom: 8px; }
.ex-card p { color: var(--text-muted); font-size: 0.9375rem; margin-bottom: 12px; line-height: 1.55; }
.ex-card .ex-meta { color: var(--text-dim); font-size: 0.875rem; font-family: var(--mono); }

.cta-banner { padding: 120px 0 100px; text-align: center; position: relative; overflow: hidden; }
.cta-banner::before { content: ''; position: absolute; inset: 0;
  background: radial-gradient(ellipse 800px 400px at center, var(--accent-glow), transparent 70%);
  pointer-events: none; }
.cta-banner > * { position: relative; }
.cta-banner h2 { font-size: clamp(2rem, 4vw, 2.8rem); max-width: 640px;
  margin: 0 auto 16px; letter-spacing: -0.02em; line-height: 1.15; }
.cta-banner h2 em { font-family: var(--serif); font-style: italic;
  color: var(--accent-bright); font-weight: 400; }
.cta-banner p { color: var(--text-muted); margin: 0 auto 28px; max-width: 460px; font-size: 1.0625rem; }
.cta-install { display: inline-flex; align-items: center; gap: 14px;
  padding: 16px 22px; background: var(--bg-raised); border: 1px solid var(--border-strong);
  border-radius: 10px; font-family: var(--mono); font-size: 1rem; }
.cta-actions { display: flex; gap: 12px; justify-content: center; margin-top: 24px; flex-wrap: wrap; }
@media (max-width: 900px) { .ex-grid { grid-template-columns: 1fr; } }
```

- [ ] **Step 2: Verify and commit**

```bash
cd site && bun run dev   # eyeball
# Ctrl+C
git add site/src/pages/index.astro
git commit -m "feat(home): examples teaser and CTA banner"
```

---

### Task 3.4: Home — plugin spotlight strip (placeholder until Phase 4)

**Files:**
- Modify: `site/src/pages/index.astro`

The plugin spotlight needs `plugins.json`. That doesn't exist yet (Phase 4 builds it). For now, hard-code 4 plugins and replace with a `getCollection`-style data read after Phase 4.

- [ ] **Step 1: Insert the spotlight section between pillars and examples**

Add this section in `site/src/pages/index.astro` between `</section>` of `.pillars` and `<section class="examples-teaser">`:

```astro
<section class="plugins-strip" aria-labelledby="plug-h">
  <div class="container">
    <div class="strip-head">
      <h2 id="plug-h">31 plugins <em>and counting.</em></h2>
      <a href={`${base}plugins`} class="link">Browse the registry <span aria-hidden="true">→</span></a>
    </div>
    <ul class="plug-grid" id="plugin-spotlight">
      <li><a href={`${base}plugins/sql`} class="plug">
        <div class="plug-head"><span class="plug-name">fledge-plugin-sql</span><span class="plug-lang">rust</span></div>
        <p>Postgres/SQLite migrations + query checks.</p>
      </a></li>
      <li><a href={`${base}plugins/deps`} class="plug">
        <div class="plug-head"><span class="plug-name">fledge-plugin-deps</span><span class="plug-lang">ts</span></div>
        <p>Audit & update dependencies across the workspace.</p>
      </a></li>
      <li><a href={`${base}plugins/todo`} class="plug">
        <div class="plug-head"><span class="plug-name">fledge-plugin-todo</span><span class="plug-lang">rust</span></div>
        <p>Scan, list, and triage TODO/FIXME markers.</p>
      </a></li>
      <li><a href={`${base}plugins/coverage`} class="plug">
        <div class="plug-head"><span class="plug-name">fledge-plugin-coverage</span><span class="plug-lang">rust</span></div>
        <p>Aggregate coverage across languages, JSON-ready.</p>
      </a></li>
    </ul>
  </div>
</section>
```

Append to `<style>`:

```css
.plugins-strip { padding: 80px 0; }
.strip-head { display: flex; justify-content: space-between; align-items: end;
  margin-bottom: 32px; gap: 16px; flex-wrap: wrap; }
.strip-head h2 { font-size: clamp(1.5rem, 3vw, 2rem); letter-spacing: -0.015em; }
.strip-head h2 em { font-family: var(--serif); font-style: italic; color: var(--accent-bright); font-weight: 400; }
.strip-head .link { color: var(--accent-bright); font-size: 0.9375rem; padding: 8px 0; }
.strip-head .link:hover { text-decoration: underline; }
.plug-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; }
.plug { display: block; padding: 20px; background: var(--bg-raised);
  border: 1px solid var(--border); border-radius: 8px;
  transition: border-color .15s; color: inherit; text-decoration: none; }
.plug:hover { border-color: var(--accent); }
.plug-head { display: flex; align-items: center; justify-content: space-between;
  margin-bottom: 10px; gap: 8px; }
.plug-name { font-family: var(--mono); font-size: 0.9375rem; color: var(--text); }
.plug-lang { font-size: 0.8125rem; padding: 3px 8px; border-radius: 4px;
  background: var(--bg); color: var(--text-muted); border: 1px solid var(--border); }
.plug p { color: var(--text-muted); font-size: 0.9375rem; line-height: 1.5; }
@media (max-width: 900px) { .plug-grid { grid-template-columns: 1fr; } }
```

- [ ] **Step 2: Build to confirm no broken imports**

```bash
cd site && bun run build
```

Expected: build succeeds. (Prebuild script not yet implemented — `bun run build` currently runs `astro build` directly, bypassing prebuild. We'll add a stub prebuild in Phase 4 Task 4.1 to keep this working.)

NOTE: If the build complains about the `prebuild` script being missing, temporarily run `bun --bun run astro build` or rename the `prebuild` key to `prebuild-disabled` until Task 4.1 wires it up. Re-rename it in Task 4.7.

- [ ] **Step 3: Commit**

```bash
git add site/src/pages/index.astro
git commit -m "feat(home): plugin spotlight strip (hardcoded for now)"
```

**🛑 Checkpoint:** Phase 3 done. The home page matches the mockup. Visually verify by running `bun run dev` and comparing to the v3 mockup in `.superpowers/brainstorm/.../home-v3.html`.

---

## Phase 4 — Plugin registry pipeline

**Goal of phase:** `bun scripts/build-plugin-registry.ts` writes `site/src/data/plugins.json` and `site/src/data/plugins/{slug}.json` from a live GitHub query, with full unit-test coverage of pure helpers.

### Task 4.1: Pure plugin helpers + tests

**Files:**
- Create: `site/scripts/plugin-helpers.ts`
- Create: `site/scripts/plugin-helpers.test.ts`
- Create: `site/scripts/community-allowlist.json`

- [ ] **Step 1: Write the failing tests first**

Create `site/scripts/plugin-helpers.test.ts`:

```ts
import { describe, test, expect } from 'bun:test'
import { slugFromName, inferLanguage, inferTrustTier } from './plugin-helpers'

describe('slugFromName', () => {
  test('strips fledge-plugin- prefix', () => {
    expect(slugFromName('fledge-plugin-sql')).toBe('sql')
  })
  test('handles multi-word names', () => {
    expect(slugFromName('fledge-plugin-todo-scan')).toBe('todo-scan')
  })
  test('returns the name unchanged if no prefix', () => {
    expect(slugFromName('fledge-deploy')).toBe('fledge-deploy')
  })
  test('throws on empty input', () => {
    expect(() => slugFromName('')).toThrow()
  })
})

describe('inferLanguage', () => {
  test('Cargo.toml → rust', () => {
    expect(inferLanguage(['README.md', 'Cargo.toml', 'src/main.rs'])).toBe('rust')
  })
  test('package.json → ts when tsconfig present', () => {
    expect(inferLanguage(['README.md', 'package.json', 'tsconfig.json'])).toBe('ts')
  })
  test('package.json without tsconfig → js', () => {
    expect(inferLanguage(['README.md', 'package.json'])).toBe('js')
  })
  test('go.mod → go', () => {
    expect(inferLanguage(['README.md', 'go.mod'])).toBe('go')
  })
  test('pyproject.toml → python', () => {
    expect(inferLanguage(['README.md', 'pyproject.toml'])).toBe('python')
  })
  test('only shell files → shell', () => {
    expect(inferLanguage(['README.md', 'install.sh'])).toBe('shell')
  })
  test('unknown → other', () => {
    expect(inferLanguage(['README.md'])).toBe('other')
  })
})

describe('inferTrustTier', () => {
  test('CorvidLabs owner → official', () => {
    expect(inferTrustTier('CorvidLabs', [])).toBe('official')
  })
  test('non-CorvidLabs with no experimental topic → community', () => {
    expect(inferTrustTier('alice', ['cli', 'rust'])).toBe('community')
  })
  test('experimental topic → experimental regardless of owner', () => {
    expect(inferTrustTier('CorvidLabs', ['fledge-plugin-experimental'])).toBe('experimental')
    expect(inferTrustTier('bob', ['fledge-plugin-experimental'])).toBe('experimental')
  })
})
```

- [ ] **Step 2: Run the tests to confirm they fail**

```bash
cd site && bun test scripts/plugin-helpers.test.ts
```

Expected: every test fails with "Cannot find module './plugin-helpers'".

- [ ] **Step 3: Implement the helpers**

Create `site/scripts/plugin-helpers.ts`:

```ts
export type Language = 'rust' | 'ts' | 'js' | 'go' | 'python' | 'shell' | 'other'
export type TrustTier = 'official' | 'community' | 'experimental'

export function slugFromName(name: string): string {
  if (!name) throw new Error('slugFromName: name is required')
  const PREFIX = 'fledge-plugin-'
  return name.startsWith(PREFIX) ? name.slice(PREFIX.length) : name
}

export function inferLanguage(repoFiles: readonly string[]): Language {
  const has = (f: string) => repoFiles.includes(f)
  if (has('Cargo.toml')) return 'rust'
  if (has('package.json')) return has('tsconfig.json') ? 'ts' : 'js'
  if (has('go.mod')) return 'go'
  if (has('pyproject.toml') || has('setup.py')) return 'python'
  if (repoFiles.some(f => f.endsWith('.sh'))) return 'shell'
  return 'other'
}

export function inferTrustTier(owner: string, topics: readonly string[]): TrustTier {
  if (topics.includes('fledge-plugin-experimental')) return 'experimental'
  if (owner === 'CorvidLabs') return 'official'
  return 'community'
}
```

- [ ] **Step 4: Re-run tests to confirm they pass**

```bash
bun test scripts/plugin-helpers.test.ts
```

Expected: all green.

- [ ] **Step 5: Add the empty community allowlist**

Create `site/scripts/community-allowlist.json`:

```json
[]
```

- [ ] **Step 6: Commit**

```bash
git add site/scripts/plugin-helpers.ts site/scripts/plugin-helpers.test.ts \
        site/scripts/community-allowlist.json
git commit -m "feat(site/scripts): pure plugin helpers + tests"
```

---

### Task 4.2: README markdown → sanitized HTML

**Files:**
- Create: `site/scripts/render-readme.ts`
- Create: `site/scripts/render-readme.test.ts`

- [ ] **Step 1: Failing test**

Create `site/scripts/render-readme.test.ts`:

```ts
import { describe, test, expect } from 'bun:test'
import { renderReadme } from './render-readme'

describe('renderReadme', () => {
  test('renders headings and paragraphs', () => {
    const html = renderReadme('# Title\n\nHello *world*.')
    expect(html).toContain('<h1>Title</h1>')
    expect(html).toContain('<em>world</em>')
  })

  test('strips inline script tags', () => {
    const html = renderReadme('<script>alert(1)</script>\nHello.')
    expect(html).not.toContain('<script')
    expect(html).toContain('Hello.')
  })

  test('strips on* event handlers', () => {
    const html = renderReadme('<a href="#" onclick="alert(1)">x</a>')
    expect(html).not.toContain('onclick')
  })

  test('keeps code fences with language', () => {
    const html = renderReadme('```rust\nfn main() {}\n```')
    expect(html).toContain('<pre>')
    expect(html).toContain('<code')
    expect(html).toContain('fn main() {}')
  })

  test('returns empty string for null/empty input', () => {
    expect(renderReadme('')).toBe('')
    expect(renderReadme(null as unknown as string)).toBe('')
  })
})
```

- [ ] **Step 2: Run, confirm fail**

```bash
bun test scripts/render-readme.test.ts
```

Expected: fails with "Cannot find module".

- [ ] **Step 3: Implement**

Create `site/scripts/render-readme.ts`:

```ts
import { marked } from 'marked'
import DOMPurify from 'isomorphic-dompurify'

export function renderReadme(markdown: string | null | undefined): string {
  if (!markdown) return ''
  const rawHtml = marked.parse(markdown, { async: false }) as string
  return DOMPurify.sanitize(rawHtml, {
    USE_PROFILES: { html: true },
    FORBID_TAGS: ['style'],
    FORBID_ATTR: ['style'],
  })
}
```

- [ ] **Step 4: Re-run tests**

```bash
bun test scripts/render-readme.test.ts
```

Expected: all green.

- [ ] **Step 5: Commit**

```bash
git add site/scripts/render-readme.ts site/scripts/render-readme.test.ts
git commit -m "feat(site/scripts): markdown→sanitized-HTML readme renderer + tests"
```

---

### Task 4.3: Related-plugin computation

**Files:**
- Create: `site/scripts/related-plugins.ts`
- Create: `site/scripts/related-plugins.test.ts`

- [ ] **Step 1: Failing test**

Create `site/scripts/related-plugins.test.ts`:

```ts
import { describe, test, expect } from 'bun:test'
import { relatedSlugs } from './related-plugins'

type Mini = { slug: string; language: string; topics: string[] }

const universe: Mini[] = [
  { slug: 'sql',      language: 'rust', topics: ['database', 'postgres'] },
  { slug: 'coverage', language: 'rust', topics: ['testing', 'coverage'] },
  { slug: 'bench',    language: 'rust', topics: ['testing', 'benchmarks'] },
  { slug: 'todo',     language: 'rust', topics: ['triage', 'codebase'] },
  { slug: 'deps',     language: 'ts',   topics: ['dependencies'] },
]

describe('relatedSlugs', () => {
  test('prefers shared topics', () => {
    const result = relatedSlugs('coverage', universe, 3)
    expect(result[0]).toBe('bench')   // shares "testing"
  })

  test('falls back to same-language when no topic overlap', () => {
    const result = relatedSlugs('todo', universe, 3)
    expect(result.length).toBe(3)
    expect(result).not.toContain('todo')          // never include self
    expect(result.every(s => s !== 'deps')).toBe(true)  // prefer same-language
  })

  test('returns at most `limit` entries', () => {
    expect(relatedSlugs('sql', universe, 2).length).toBe(2)
  })

  test('returns empty when only one plugin exists', () => {
    expect(relatedSlugs('sql', [universe[0]], 3)).toEqual([])
  })
})
```

- [ ] **Step 2: Run, confirm fail**

- [ ] **Step 3: Implement**

Create `site/scripts/related-plugins.ts`:

```ts
export interface MiniPlugin {
  slug: string
  language: string
  topics: readonly string[]
}

export function relatedSlugs<T extends MiniPlugin>(
  slug: string,
  universe: readonly T[],
  limit: number,
): string[] {
  const self = universe.find(p => p.slug === slug)
  if (!self) return []
  const others = universe.filter(p => p.slug !== slug)
  if (others.length === 0) return []

  const score = (p: T): number => {
    const sharedTopics = p.topics.filter(t => self.topics.includes(t)).length
    const langBonus = p.language === self.language ? 0.5 : 0
    return sharedTopics + langBonus
  }

  return others
    .map(p => ({ p, s: score(p) }))
    .sort((a, b) => b.s - a.s || a.p.slug.localeCompare(b.p.slug))
    .slice(0, limit)
    .map(x => x.p.slug)
}
```

- [ ] **Step 4: Re-run tests**

```bash
bun test scripts/related-plugins.test.ts
```

Expected: green.

- [ ] **Step 5: Commit**

```bash
git add site/scripts/related-plugins.ts site/scripts/related-plugins.test.ts
git commit -m "feat(site/scripts): related-plugin ranking by topic overlap + lang"
```

---

### Task 4.4: GitHub fetch — the registry builder (with cache fallback)

**Files:**
- Create: `site/scripts/build-plugin-registry.ts`
- Create: `site/scripts/build-plugin-registry.test.ts`
- Create: `site/src/data/plugins.json` (seed; lets dev server run before first fetch)
- Create: `site/src/data/plugins/.gitkeep`

This script is I/O-heavy. The unit test covers the pure transformation `repoToEntry()`. The fetch itself is covered by an integration step (run it locally; CI runs it on every build).

- [ ] **Step 1: Failing test for the pure repo→entry transform**

Create `site/scripts/build-plugin-registry.test.ts`:

```ts
import { describe, test, expect } from 'bun:test'
import { repoToEntry } from './build-plugin-registry'

const sampleRepo = {
  name: 'fledge-plugin-sql',
  owner: { login: 'CorvidLabs' },
  description: 'Postgres + SQLite migrations',
  html_url: 'https://github.com/CorvidLabs/fledge-plugin-sql',
  default_branch: 'main',
  stargazers_count: 142,
  topics: ['database', 'postgres'],
  pushed_at: '2026-04-01T12:00:00Z',
  license: { spdx_id: 'MIT' },
  open_issues_count: 3,
}

describe('repoToEntry', () => {
  test('produces a registry entry from a GitHub repo + manifest data', () => {
    const entry = repoToEntry(sampleRepo, {
      files: ['Cargo.toml', 'README.md', 'src/main.rs'],
      version: '0.3.0',
    })
    expect(entry.name).toBe('fledge-plugin-sql')
    expect(entry.slug).toBe('sql')
    expect(entry.version).toBe('0.3.0')
    expect(entry.description).toBe('Postgres + SQLite migrations')
    expect(entry.language).toBe('rust')
    expect(entry.trust_tier).toBe('official')
    expect(entry.install).toBe('fledge plugins install CorvidLabs/fledge-plugin-sql')
    expect(entry.repo).toBe('https://github.com/CorvidLabs/fledge-plugin-sql')
    expect(entry.topics).toEqual(['database', 'postgres'])
    expect(entry.stars).toBe(142)
    expect(entry.updated_at).toBe('2026-04-01T12:00:00Z')
    expect(entry.default_branch).toBe('main')
  })

  test('falls back to "unknown" version when no manifest version is given', () => {
    const entry = repoToEntry(sampleRepo, { files: ['Cargo.toml'], version: null })
    expect(entry.version).toBe('unknown')
  })
})
```

- [ ] **Step 2: Implement the script (and re-export `repoToEntry`)**

Create `site/scripts/build-plugin-registry.ts`:

```ts
import { writeFileSync, mkdirSync, readFileSync, existsSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { slugFromName, inferLanguage, inferTrustTier, type Language, type TrustTier } from './plugin-helpers'
import { renderReadme } from './render-readme'
import { relatedSlugs } from './related-plugins'
import allowlist from './community-allowlist.json' with { type: 'json' }

const __dirname = dirname(fileURLToPath(import.meta.url))
const DATA_DIR = join(__dirname, '..', 'src', 'data')
const PER_PLUGIN_DIR = join(DATA_DIR, 'plugins')
const INDEX_PATH = join(DATA_DIR, 'plugins.json')
const TOKEN = process.env.GITHUB_TOKEN

export interface RegistryEntry {
  name: string
  slug: string
  version: string
  description: string
  language: Language
  trust_tier: TrustTier
  install: string
  repo: string
  topics: string[]
  stars: number
  updated_at: string
  default_branch: string
}

export interface FullEntry extends RegistryEntry {
  readme_html: string
  license: string | null
  open_issues: number
  related_slugs: string[]
}

interface GhRepo {
  name: string
  owner: { login: string }
  description: string | null
  html_url: string
  default_branch: string
  stargazers_count: number
  topics: string[]
  pushed_at: string
  license: { spdx_id: string } | null
  open_issues_count: number
}

interface ManifestInfo { files: string[]; version: string | null }

export function repoToEntry(repo: GhRepo, info: ManifestInfo): RegistryEntry {
  return {
    name: repo.name,
    slug: slugFromName(repo.name),
    version: info.version ?? 'unknown',
    description: repo.description ?? '',
    language: inferLanguage(info.files),
    trust_tier: inferTrustTier(repo.owner.login, repo.topics ?? []),
    install: `fledge plugins install ${repo.owner.login}/${repo.name}`,
    repo: repo.html_url,
    topics: repo.topics ?? [],
    stars: repo.stargazers_count,
    updated_at: repo.pushed_at,
    default_branch: repo.default_branch,
  }
}

async function gh<T>(url: string): Promise<T> {
  const headers: Record<string, string> = {
    Accept: 'application/vnd.github+json',
    'User-Agent': 'fledge-site-builder',
  }
  if (TOKEN) headers.Authorization = `Bearer ${TOKEN}`
  const res = await fetch(url, { headers })
  if (!res.ok) throw new Error(`GH ${res.status} ${res.statusText} on ${url}`)
  return res.json() as Promise<T>
}

async function listFledgePluginRepos(): Promise<GhRepo[]> {
  const owners = ['CorvidLabs', ...(allowlist as string[])]
  const all: GhRepo[] = []
  for (const owner of owners) {
    const q = encodeURIComponent(`org:${owner} fledge-plugin- in:name`)
    const url = `https://api.github.com/search/repositories?q=${q}&per_page=100`
    const result = await gh<{ items: GhRepo[] }>(url)
    for (const item of result.items) {
      if (item.name.startsWith('fledge-plugin-')) all.push(item)
    }
  }
  return all
}

async function fetchManifest(owner: string, repo: string, branch: string): Promise<ManifestInfo> {
  const listing = await gh<{ tree: { path: string; type: string }[] }>(
    `https://api.github.com/repos/${owner}/${repo}/git/trees/${branch}?recursive=0`,
  )
  const files = listing.tree.filter(n => n.type === 'blob').map(n => n.path)
  let version: string | null = null
  if (files.includes('Cargo.toml')) {
    const cargoToml = await fetch(
      `https://raw.githubusercontent.com/${owner}/${repo}/${branch}/Cargo.toml`,
    ).then(r => r.text()).catch(() => '')
    const m = cargoToml.match(/^version\s*=\s*"([^"]+)"/m)
    version = m?.[1] ?? null
  } else if (files.includes('package.json')) {
    const pkg = await fetch(
      `https://raw.githubusercontent.com/${owner}/${repo}/${branch}/package.json`,
    ).then(r => r.json()).catch(() => ({}))
    version = (pkg as { version?: string }).version ?? null
  }
  return { files, version }
}

async function fetchReadme(owner: string, repo: string, branch: string): Promise<string> {
  for (const name of ['README.md', 'readme.md', 'README.MD', 'README']) {
    const res = await fetch(`https://raw.githubusercontent.com/${owner}/${repo}/${branch}/${name}`)
    if (res.ok) return res.text()
  }
  return ''
}

function loadCachedIndex(): RegistryEntry[] | null {
  if (!existsSync(INDEX_PATH)) return null
  try { return JSON.parse(readFileSync(INDEX_PATH, 'utf-8')) as RegistryEntry[] }
  catch { return null }
}

async function main() {
  mkdirSync(PER_PLUGIN_DIR, { recursive: true })

  let repos: GhRepo[]
  try {
    repos = await listFledgePluginRepos()
  } catch (e) {
    console.warn(`[build-plugin-registry] GH fetch failed: ${(e as Error).message}`)
    const cached = loadCachedIndex()
    if (cached) {
      console.warn('[build-plugin-registry] using cached plugins.json from previous build')
      return
    }
    console.error('[build-plugin-registry] no cache available, writing empty index')
    writeFileSync(INDEX_PATH, '[]')
    return
  }

  const enrich = await Promise.all(repos.map(async r => {
    const info = await fetchManifest(r.owner.login, r.name, r.default_branch).catch(() => ({ files: [], version: null }))
    const readme = await fetchReadme(r.owner.login, r.name, r.default_branch).catch(() => '')
    return { repo: r, info, readme }
  }))

  const index: RegistryEntry[] = enrich.map(({ repo, info }) => repoToEntry(repo, info))
  const miniUniverse = index.map(e => ({ slug: e.slug, language: e.language, topics: e.topics }))

  writeFileSync(INDEX_PATH, JSON.stringify(index, null, 2))
  console.log(`[build-plugin-registry] wrote ${index.length} entries to plugins.json`)

  for (const { repo, info, readme } of enrich) {
    const base = repoToEntry(repo, info)
    const full: FullEntry = {
      ...base,
      readme_html: renderReadme(readme),
      license: repo.license?.spdx_id ?? null,
      open_issues: repo.open_issues_count,
      related_slugs: relatedSlugs(base.slug, miniUniverse, 3),
    }
    writeFileSync(join(PER_PLUGIN_DIR, `${base.slug}.json`), JSON.stringify(full, null, 2))
  }
  console.log(`[build-plugin-registry] wrote ${enrich.length} per-plugin files`)
}

// Only run when invoked as a script, not when imported by tests.
if (import.meta.main) {
  main().catch(err => { console.error(err); process.exit(1) })
}
```

- [ ] **Step 3: Run the unit test**

```bash
cd site && bun test scripts/build-plugin-registry.test.ts
```

Expected: green.

- [ ] **Step 4: Seed the empty index so dev server runs without a network fetch**

Create `site/src/data/plugins.json`:

```json
[]
```

Create `site/src/data/plugins/.gitkeep` (empty file).

- [ ] **Step 5: Commit**

```bash
git add site/scripts/build-plugin-registry.ts site/scripts/build-plugin-registry.test.ts \
        site/src/data/plugins.json site/src/data/plugins/.gitkeep
git commit -m "feat(site/scripts): build-plugin-registry with cache fallback"
```

---

### Task 4.5: Run the fetcher end-to-end (integration smoke)

This is a one-shot manual test — we run it locally to confirm it produces real data, then move on.

- [ ] **Step 1: Run the fetcher**

```bash
cd site && GITHUB_TOKEN=$(gh auth token) bun scripts/build-plugin-registry.ts
```

Expected output:
- A line like `[build-plugin-registry] wrote NN entries to plugins.json`
- A line like `[build-plugin-registry] wrote NN per-plugin files`
- `site/src/data/plugins.json` is now an array of registry entries
- `site/src/data/plugins/sql.json` (etc) exist

- [ ] **Step 2: Spot-check the output**

Look at one entry: `cat site/src/data/plugins/sql.json | jq '{name, slug, version, language, trust_tier, related_slugs}'`

If anything looks wrong (missing fields, bogus values), fix the script before continuing.

- [ ] **Step 3: Revert the generated data so the commit stays clean**

The generated files are gitignored, so nothing to commit. But verify:

```bash
git status site/src/data/
```

Expected: nothing to commit (the JSON files are gitignored).

---

### Task 4.6: Wire prebuild and add the build-time tests to the test script

**Files:**
- Modify: `site/package.json`

- [ ] **Step 1: Confirm package.json prebuild line is correct**

It should already say:

```json
"prebuild": "bun scripts/build-plugin-registry.ts && bun scripts/generate-doc-redirects.ts",
```

`generate-doc-redirects.ts` doesn't exist yet (Phase 9). Comment it out for now:

```json
"prebuild": "bun scripts/build-plugin-registry.ts",
```

- [ ] **Step 2: Run a full build**

```bash
cd site && GITHUB_TOKEN=$(gh auth token) bun run build
```

Expected: prebuild runs, then astro build runs, then `site/dist/` is populated. No errors.

- [ ] **Step 3: Commit**

```bash
git add site/package.json
git commit -m "feat(site): wire build-plugin-registry into prebuild"
```

**🛑 Checkpoint:** Phase 4 done. `bun run build` produces a populated `dist/`.

---

## Phase 5 — Plugins pages

**Goal of phase:** `/plugins` shows the searchable registry; `/plugins/{slug}` renders per-plugin pages with the README.

### Task 5.1: PluginCard component

**Files:**
- Create: `site/src/components/PluginCard.astro`

- [ ] **Step 1: Implement**

Create `site/src/components/PluginCard.astro`:

```astro
---
import type { RegistryEntry } from '../../scripts/build-plugin-registry'
interface Props { plugin: RegistryEntry }
const { plugin } = Astro.props
const base = import.meta.env.BASE_URL
const tierClass = `tag tag-tier-${plugin.trust_tier}`
const tierLabel = plugin.trust_tier.charAt(0).toUpperCase() + plugin.trust_tier.slice(1)
---
<li><a href={`${base}plugins/${plugin.slug}`} class="plugin-card">
  <div class="plug-top">
    <div class="plug-name">{plugin.name}</div>
    <div class="plug-version">v{plugin.version}</div>
  </div>
  <div class="plug-tags">
    <span class={tierClass}>{tierLabel}</span>
    <span class="tag tag-lang">{plugin.language}</span>
  </div>
  <p class="plug-desc">{plugin.description || 'No description.'}</p>
  <div class="plug-foot">
    <code class="plug-install">{plugin.install.replace(/^fledge plugins install /, '… install ')}</code>
    <div class="plug-meta-mini">★ {plugin.stars}</div>
  </div>
</a></li>

<style is:global>
  .plugin-card { display: flex; flex-direction: column; gap: 14px;
    padding: 24px; background: var(--bg-raised);
    border: 1px solid var(--border); border-radius: var(--radius);
    transition: border-color .15s, transform .15s; color: inherit; text-decoration: none; }
  .plugin-card:hover { border-color: var(--accent); transform: translateY(-2px); }
  .plug-top { display: flex; justify-content: space-between; align-items: flex-start; gap: 12px; }
  .plug-name { font-family: var(--mono); font-size: 1rem; color: var(--text); font-weight: 500; word-break: break-all; }
  .plug-version { font-family: var(--mono); font-size: 0.8125rem; color: var(--text-dim); flex-shrink: 0; }
  .plug-tags { display: flex; gap: 6px; flex-wrap: wrap; }
  .tag-tier-official    { background: rgba(52,211,153,0.1); color: #34d399; border: 1px solid rgba(52,211,153,0.3); }
  .tag-tier-community   { background: rgba(96,165,250,0.1); color: #60a5fa; border: 1px solid rgba(96,165,250,0.3); }
  .tag-tier-experimental{ background: rgba(251,191,36,0.1); color: #fbbf24; border: 1px solid rgba(251,191,36,0.3); }
  .tag-lang { background: transparent; color: var(--text-muted);
    border: 1px solid var(--border-strong); text-transform: lowercase; letter-spacing: 0; }
  .plug-desc { color: var(--text-muted); font-size: 0.9375rem; line-height: 1.55; flex: 1; }
  .plug-foot { display: flex; justify-content: space-between; align-items: center;
    gap: 12px; padding-top: 12px; border-top: 1px dashed var(--border-strong); margin-top: 4px; }
  .plug-install { font-family: var(--mono); font-size: 0.8125rem; color: var(--accent-bright);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .plug-meta-mini { color: var(--text-dim); font-size: 0.8125rem; display: flex; gap: 10px; flex-shrink: 0; }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add site/src/components/PluginCard.astro
git commit -m "feat(site/components): PluginCard"
```

---

### Task 5.2: Plugins index page

**Files:**
- Create: `site/src/pages/plugins/index.astro`

- [ ] **Step 1: Implement (with client-side search/filter)**

Create `site/src/pages/plugins/index.astro`:

```astro
---
import BaseLayout from '../../layouts/BaseLayout.astro'
import PluginCard from '../../components/PluginCard.astro'
import plugins from '../../data/plugins.json' with { type: 'json' }
import type { RegistryEntry } from '../../../scripts/build-plugin-registry'
const list = plugins as RegistryEntry[]
const officialCount = list.filter(p => p.trust_tier === 'official').length
const communityCount = list.filter(p => p.trust_tier === 'community').length
const langCounts = list.reduce<Record<string, number>>((acc, p) => {
  acc[p.language] = (acc[p.language] ?? 0) + 1
  return acc
}, {})
---
<BaseLayout title="fledge — plugin registry">
<main id="main">

<section class="page-head">
  <div class="container">
    <p class="eyebrow">Plugin Registry</p>
    <h1>Extend fledge with <em>one command.</em></h1>
    <p class="lede">Browse {list.length} plugins maintained by CorvidLabs and the community. Install any of them in one line.</p>
    <div class="page-stats">
      <span><strong>{list.length}</strong> plugins</span>
      <span><strong>{officialCount}</strong> official</span>
      <span><strong>{communityCount}</strong> community</span>
      <span>Auto-refreshed weekly</span>
    </div>
  </div>
</section>

<section class="registry">
  <div class="container">

    <div class="controls" role="search">
      <div class="search-wrap">
        <span class="icon" aria-hidden="true">⌕</span>
        <label for="q" class="sr-only">Search plugins</label>
        <input type="search" id="q" class="search" placeholder="Search by name, description, or topic…" />
      </div>
      <div class="filter-group" role="group" aria-label="Trust tier">
        <span class="filter-label">Tier:</span>
        <button class="chip active" type="button" data-tier="" aria-pressed="true">All <span class="count">{list.length}</span></button>
        <button class="chip" type="button" data-tier="official" aria-pressed="false">Official <span class="count">{officialCount}</span></button>
        <button class="chip" type="button" data-tier="community" aria-pressed="false">Community <span class="count">{communityCount}</span></button>
      </div>
      <div class="filter-group" role="group" aria-label="Language">
        <span class="filter-label">Lang:</span>
        <button class="chip active" type="button" data-lang="" aria-pressed="true">Any</button>
        {Object.entries(langCounts).map(([lang, n]) => (
          <button class="chip" type="button" data-lang={lang} aria-pressed="false">{lang} <span class="count">{n}</span></button>
        ))}
      </div>
    </div>

    <p id="result-meta" class="results-meta">Showing <strong>{list.length}</strong> of <strong>{list.length}</strong> plugins</p>

    <ul class="plugin-grid" id="grid">
      {list.map(p => <PluginCard plugin={p} />)}
    </ul>

  </div>
</section>

</main>
</BaseLayout>

<script>
  const $ = (sel: string) => document.querySelector(sel)
  const $$ = (sel: string) => Array.from(document.querySelectorAll(sel))
  const grid = $('#grid') as HTMLUListElement
  const search = $('#q') as HTMLInputElement
  const meta = $('#result-meta') as HTMLElement
  const cards = $$('#grid > li') as HTMLLIElement[]
  let activeTier = ''
  let activeLang = ''
  let activeQuery = ''

  function apply() {
    let visible = 0
    cards.forEach(li => {
      const text = li.textContent?.toLowerCase() ?? ''
      const tier = (li.querySelector('[class*="tag-tier-"]')?.className.match(/tag-tier-(\w+)/)?.[1]) ?? ''
      const lang = li.querySelector('.tag-lang')?.textContent?.trim() ?? ''
      const matchTier = !activeTier || tier === activeTier
      const matchLang = !activeLang || lang === activeLang
      const matchQuery = !activeQuery || text.includes(activeQuery)
      const show = matchTier && matchLang && matchQuery
      li.style.display = show ? '' : 'none'
      if (show) visible++
    })
    meta.innerHTML = `Showing <strong>${visible}</strong> of <strong>${cards.length}</strong> plugins`
  }

  search?.addEventListener('input', () => {
    activeQuery = search.value.toLowerCase()
    apply()
  })

  $$('[data-tier]').forEach(btn => btn.addEventListener('click', () => {
    activeTier = (btn as HTMLButtonElement).dataset.tier ?? ''
    $$('[data-tier]').forEach(b => {
      b.classList.toggle('active', b === btn)
      b.setAttribute('aria-pressed', String(b === btn))
    })
    apply()
  }))

  $$('[data-lang]').forEach(btn => btn.addEventListener('click', () => {
    activeLang = (btn as HTMLButtonElement).dataset.lang ?? ''
    $$('[data-lang]').forEach(b => {
      b.classList.toggle('active', b === btn)
      b.setAttribute('aria-pressed', String(b === btn))
    })
    apply()
  }))
</script>

<style>
  .page-head { padding: 60px 0 40px; background-image: radial-gradient(ellipse 1200px 400px at 50% -100px, rgba(234,88,12,0.10), transparent 70%); border-bottom: 1px solid var(--border); text-align: center; }
  .eyebrow { color: var(--accent-bright); font-size: 0.875rem; font-weight: 600; letter-spacing: 0.12em; text-transform: uppercase; margin-bottom: 14px; }
  .page-head h1 { font-size: clamp(2.2rem, 4vw, 3rem); letter-spacing: -0.025em; margin-bottom: 14px; line-height: 1.1; }
  .page-head h1 em { font-style: italic; font-family: var(--serif); color: var(--accent-bright); font-weight: 400; }
  .lede { color: var(--text-muted); font-size: 1.125rem; max-width: 540px; margin: 0 auto 32px; }
  .page-stats { display: flex; gap: 24px; justify-content: center; flex-wrap: wrap; color: var(--text-muted); font-size: 0.9375rem; }
  .page-stats strong { color: var(--text); font-weight: 600; }

  .registry { padding: 40px 0 80px; }
  .controls { display: flex; gap: 16px; align-items: center; margin-bottom: 28px; flex-wrap: wrap;
    padding: 16px; background: var(--bg-raised); border: 1px solid var(--border); border-radius: var(--radius); }
  .search-wrap { flex: 1 1 320px; position: relative; min-width: 260px; }
  .icon { position: absolute; left: 14px; top: 50%; transform: translateY(-50%); color: var(--text-muted); font-size: 1.1rem; pointer-events: none; }
  .search { width: 100%; padding: 14px 14px 14px 42px; background: var(--bg);
    border: 1px solid var(--border-strong); border-radius: 8px;
    color: var(--text); font-size: 1rem; font-family: inherit; min-height: 44px; }
  .search::placeholder { color: var(--text-dim); }
  .filter-group { display: flex; gap: 6px; }
  .filter-label { color: var(--text-dim); font-size: 0.875rem; padding: 8px 0; align-self: center; }
  .chip { padding: 8px 14px; border-radius: 999px; background: transparent;
    border: 1px solid var(--border-strong); color: var(--text-muted);
    font-size: 0.875rem; font-family: inherit; min-height: 36px; }
  .chip:hover { color: var(--text); border-color: var(--accent); }
  .chip.active { background: var(--accent-muted); border-color: var(--accent); color: var(--accent-bright); }
  .chip .count { color: var(--text-dim); margin-left: 6px; font-size: 0.8125rem; }
  .results-meta { color: var(--text-muted); font-size: 0.9375rem; margin-bottom: 16px; }
  .results-meta strong { color: var(--text); }
  .plugin-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; }
  @media (max-width: 1024px) { .plugin-grid { grid-template-columns: repeat(2, 1fr); } }
  @media (max-width: 700px) { .plugin-grid { grid-template-columns: 1fr; } .controls { flex-direction: column; align-items: stretch; } }
</style>
```

- [ ] **Step 2: Build to verify**

```bash
cd site && GITHUB_TOKEN=$(gh auth token) bun run build
```

Expected: build succeeds. Open `site/dist/plugins/index.html` in a browser (or `bun run preview` and visit `/fledge/plugins`).

- [ ] **Step 3: Commit**

```bash
git add site/src/pages/plugins/index.astro
git commit -m "feat(plugins): registry index with client-side search and filters"
```

---

### Task 5.3: Per-plugin page

**Files:**
- Create: `site/src/pages/plugins/[slug].astro`

- [ ] **Step 1: Implement**

Create `site/src/pages/plugins/[slug].astro`:

```astro
---
import { readdirSync, readFileSync, existsSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import BaseLayout from '../../layouts/BaseLayout.astro'
import Button from '../../components/Button.astro'
import type { FullEntry } from '../../../scripts/build-plugin-registry'

const __dirname = dirname(fileURLToPath(import.meta.url))
const PER_PLUGIN_DIR = join(__dirname, '..', '..', 'data', 'plugins')

export async function getStaticPaths() {
  const dir = join(dirname(fileURLToPath(import.meta.url)), '..', '..', 'data', 'plugins')
  if (!existsSync(dir)) return []
  return readdirSync(dir)
    .filter(f => f.endsWith('.json'))
    .map(f => {
      const slug = f.replace(/\.json$/, '')
      const data = JSON.parse(readFileSync(join(dir, f), 'utf-8')) as FullEntry
      return { params: { slug }, props: { plugin: data } }
    })
}

interface Props { plugin: FullEntry }
const { plugin } = Astro.props
const base = import.meta.env.BASE_URL

// load related plugins from disk so we have title + description for cards
const related = plugin.related_slugs
  .map(s => {
    const p = join(PER_PLUGIN_DIR, `${s}.json`)
    return existsSync(p) ? (JSON.parse(readFileSync(p, 'utf-8')) as FullEntry) : null
  })
  .filter((x): x is FullEntry => !!x)
---
<BaseLayout title={`${plugin.name} — fledge plugin`} description={plugin.description}>
<main id="main">

<section class="plug-head">
  <div class="container">
    <a href={`${base}plugins`} class="back-link"><span aria-hidden="true">←</span> Back to plugins</a>
    <h1>{plugin.name}</h1>
    <div class="plug-meta">
      <span class="plug-version">v{plugin.version}</span>
      <span class={`tag tag-tier-${plugin.trust_tier}`}>{plugin.trust_tier}</span>
      <span class="tag tag-lang">{plugin.language}</span>
      <span class="dim">★ {plugin.stars}</span>
      <span class="dim">Updated {new Date(plugin.updated_at).toLocaleDateString()}</span>
      <a href={plugin.repo} target="_blank" rel="noopener">GitHub <span aria-hidden="true">↗</span></a>
    </div>
  </div>
</section>

<section class="install">
  <div class="container">
    <div class="install-card">
      <div class="install-label">Install with fledge</div>
      <div class="install-cmd"><code>{plugin.install}</code></div>
    </div>
  </div>
</section>

<section class="plug-body">
  <div class="container plug-grid">
    <article class="readme">
      {plugin.readme_html
        ? <Fragment set:html={plugin.readme_html} />
        : <p class="dim"><em>No README yet.</em></p>}
    </article>
    <aside class="plug-side">
      <h2>Metadata</h2>
      <dl>
        <dt>License</dt><dd>{plugin.license ?? '—'}</dd>
        <dt>Default branch</dt><dd>{plugin.default_branch}</dd>
        <dt>Open issues</dt><dd>{plugin.open_issues}</dd>
        <dt>Topics</dt><dd>{plugin.topics.length ? plugin.topics.join(', ') : '—'}</dd>
      </dl>
    </aside>
  </div>
</section>

{related.length > 0 && (
  <section class="related">
    <div class="container">
      <h2>Related plugins</h2>
      <ul class="rel-grid">
        {related.map(r => (
          <li><a href={`${base}plugins/${r.slug}`} class="rel-card">
            <div class="rel-name">{r.name}</div>
            <div class="rel-desc">{r.description}</div>
          </a></li>
        ))}
      </ul>
    </div>
  </section>
)}

<section class="cta-thin">
  <div class="container" style="text-align: center;">
    <p style="color: var(--text-muted); margin-bottom: 16px;">Built something similar?</p>
    <Button href={`${base}docs/template-authoring`} variant="primary">Submit your plugin</Button>
  </div>
</section>

</main>
</BaseLayout>

<style>
  .plug-head { padding: 48px 0 24px; border-bottom: 1px solid var(--border); }
  .back-link { color: var(--text-muted); font-size: 0.9375rem; display: inline-block; margin-bottom: 16px; }
  .back-link:hover { color: var(--accent-bright); }
  .plug-head h1 { font-family: var(--mono); font-size: 2rem; margin-bottom: 12px; word-break: break-all; }
  .plug-meta { display: flex; gap: 14px; align-items: center; flex-wrap: wrap; font-size: 0.9375rem; }
  .plug-meta .dim { color: var(--text-dim); }
  .plug-meta a { color: var(--accent-bright); }
  .plug-meta a:hover { text-decoration: underline; }
  .plug-version { font-family: var(--mono); color: var(--text-dim); font-size: 0.875rem; }
  .install { padding: 24px 0; }
  .install-card { padding: 20px 24px; background: var(--bg-raised); border: 1px solid var(--border-strong); border-radius: var(--radius); }
  .install-label { color: var(--text-dim); font-size: 0.875rem; margin-bottom: 8px; }
  .install-cmd code { font-family: var(--mono); color: var(--accent-bright); font-size: 1rem; }
  .plug-body { padding: 32px 0 60px; }
  .plug-grid { display: grid; grid-template-columns: 1fr 240px; gap: 40px; align-items: start; }
  .readme { color: var(--text); line-height: 1.7; }
  .readme :global(h1) { font-size: 1.8rem; margin: 24px 0 12px; }
  .readme :global(h2) { font-size: 1.4rem; margin: 24px 0 12px; padding-bottom: 6px; border-bottom: 1px solid var(--border); }
  .readme :global(h3) { font-size: 1.15rem; margin: 20px 0 10px; }
  .readme :global(p) { margin-bottom: 14px; }
  .readme :global(code) { font-family: var(--mono); font-size: 0.875em; padding: 2px 6px; background: var(--bg-raised); border: 1px solid var(--border); border-radius: 4px; color: var(--accent-bright); }
  .readme :global(pre) { background: var(--bg-raised); border: 1px solid var(--border); border-radius: 8px; padding: 16px; overflow-x: auto; margin-bottom: 16px; }
  .readme :global(pre code) { background: transparent; border: 0; padding: 0; color: var(--text); }
  .readme :global(ul), .readme :global(ol) { padding-left: 24px; margin-bottom: 14px; }
  .readme :global(a) { color: var(--accent-bright); text-decoration: underline; text-underline-offset: 3px; }
  .plug-side h2 { font-size: 0.875rem; color: var(--text-dim); letter-spacing: 0.1em; text-transform: uppercase; margin-bottom: 14px; }
  .plug-side dt { color: var(--text-dim); font-size: 0.8125rem; text-transform: uppercase; letter-spacing: 0.08em; margin-top: 12px; }
  .plug-side dd { color: var(--text); font-size: 0.9375rem; margin-bottom: 4px; }
  .plug-side dd:first-of-type { margin-top: 0; }
  .related { padding: 40px 0; border-top: 1px solid var(--border); }
  .related h2 { font-size: 1.4rem; margin-bottom: 20px; }
  .rel-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; }
  .rel-card { display: block; padding: 18px; background: var(--bg-raised); border: 1px solid var(--border); border-radius: 8px; }
  .rel-card:hover { border-color: var(--accent); }
  .rel-name { font-family: var(--mono); font-size: 0.9375rem; color: var(--text); margin-bottom: 6px; }
  .rel-desc { color: var(--text-muted); font-size: 0.875rem; }
  .cta-thin { padding: 60px 0; }
  @media (max-width: 900px) {
    .plug-grid { grid-template-columns: 1fr; }
    .rel-grid { grid-template-columns: 1fr; }
  }
</style>
```

- [ ] **Step 2: Build and verify**

```bash
cd site && GITHUB_TOKEN=$(gh auth token) bun run build
```

Open `site/dist/plugins/sql/index.html` (or whatever first slug exists) and confirm the README renders.

- [ ] **Step 3: Commit**

```bash
git add site/src/pages/plugins/[slug].astro
git commit -m "feat(plugins): per-plugin page with README, metadata, related plugins"
```

**🛑 Checkpoint:** Phase 5 done. Plugin registry browseable; each plugin has a real page.

---

## Phase 6 — Examples

**Goal of phase:** `/examples` lists walkthroughs from a content collection; `/examples/{slug}` renders the MDX.

### Task 6.1: Content collection schema

**Files:**
- Create: `site/src/content/config.ts`
- Create: `site/src/content/examples/.gitkeep`

- [ ] **Step 1: Schema**

Create `site/src/content/config.ts`:

```ts
import { defineCollection, z } from 'astro:content'

const examples = defineCollection({
  type: 'content',
  schema: z.object({
    title: z.string(),
    tag: z.enum(['Rust CLI', 'TS + Bun', 'Python', 'Go', 'Plugins', 'AI', 'CI / CD', 'Monorepo', 'Templates']),
    steps: z.number().int().positive(),
    minutes: z.number().int().positive(),
    pillars: z.array(z.string()),
    description: z.string(),
    featured: z.boolean().default(false),
    draft: z.boolean().default(false),
    order: z.number().optional(),
  }),
})

export const collections = { examples }
```

Create `site/src/content/examples/.gitkeep` (empty).

- [ ] **Step 2: Add a seed walkthrough**

Create `site/src/content/examples/rust-cli.mdx`:

```mdx
---
title: "Build a Rust CLI end-to-end"
tag: "Rust CLI"
steps: 8
minutes: 12
pillars: ["scaffold", "run", "ship"]
description: "From templates init through release bump, with conventional commits."
featured: true
order: 1
---

import Callout from '../../components/Callout.astro'

This walkthrough takes a Rust CLI from `templates init` to a tagged release in eight commands.

## 1. Scaffold

```bash
fledge templates init my-cli -t rust-cli
cd my-cli
```

(... rest of walkthrough goes here — placeholder content acceptable for v1 ...)

<Callout type="tip">
  **Tip:** `fledge lanes init` infers your build tooling from the manifest — no hand-editing needed.
</Callout>
```

Create `site/src/content/examples/ts-bun.mdx` (smaller stub):

```mdx
---
title: "TypeScript project with Bun"
tag: "TS + Bun"
steps: 6
minutes: 8
pillars: ["run", "ship"]
description: "Language detection, lane composition, ship workflow."
order: 2
---

(Walkthrough stub — fill in before launch.)
```

Create `site/src/content/examples/custom-plugin.mdx`:

```mdx
---
title: "Wire up a custom plugin"
tag: "Plugins"
steps: 10
minutes: 20
pillars: ["extend"]
description: "Build a fledge-plugin-* with the v1 protocol."
order: 3
---

(Walkthrough stub — fill in before launch.)
```

- [ ] **Step 3: Verify with astro check**

```bash
cd site && bun run lint
```

Expected: no errors. If Astro complains about `astro:content` types, run `bun run dev` once to generate `.astro/types.d.ts`, then re-run.

- [ ] **Step 4: Commit**

```bash
git add site/src/content/config.ts site/src/content/examples/
git commit -m "feat(content): examples collection schema + 3 seed walkthroughs"
```

---

### Task 6.2: Examples index page

**Files:**
- Create: `site/src/pages/examples/index.astro`

- [ ] **Step 1: Implement**

Create `site/src/pages/examples/index.astro`:

```astro
---
import { getCollection } from 'astro:content'
import BaseLayout from '../../layouts/BaseLayout.astro'
const base = import.meta.env.BASE_URL
const all = (await getCollection('examples', e => !e.data.draft)).sort(
  (a, b) => (a.data.order ?? 999) - (b.data.order ?? 999),
)
const featured = all.find(e => e.data.featured)
const rest = all.filter(e => !e.data.featured)
---
<BaseLayout title="fledge — walkthroughs and examples">
<main id="main">

<section class="page-head">
  <div class="container">
    <p class="eyebrow">Walkthroughs</p>
    <h1>fledge on a <em>real</em> project.</h1>
    <p class="lede">End-to-end walkthroughs. Every command, every file, every output. Run them yourself or read them like recipes.</p>
  </div>
</section>

{featured && (
  <section class="featured">
    <div class="container">
      <a href={`${base}examples/${featured.slug}`} class="featured-card">
        <span class="tag">{featured.data.tag}</span>
        <h2>{featured.data.title}</h2>
        <p>{featured.data.description}</p>
        <div class="featured-meta">
          <span><strong>{featured.data.steps}</strong> steps</span>
          <span><strong>{featured.data.minutes}</strong> min</span>
          <span>{featured.data.pillars.join(' · ')}</span>
        </div>
      </a>
    </div>
  </section>
)}

<section class="ex-list">
  <div class="container">
    <ul class="ex-grid">
      {rest.map((e, i) => (
        <li><a href={`${base}examples/${e.slug}`} class="ex-card">
          <div class="number">{String(i + 1).padStart(2, '0')}</div>
          <div>
            <span class="tag">{e.data.tag}</span>
            <h3>{e.data.title}</h3>
            <p>{e.data.description}</p>
            <div class="meta">{e.data.steps} steps · {e.data.minutes} min</div>
          </div>
        </a></li>
      ))}
    </ul>
  </div>
</section>

</main>
</BaseLayout>

<style>
  .page-head { padding: 60px 0 40px; background-image: radial-gradient(ellipse 1200px 400px at 50% -100px, rgba(234,88,12,0.10), transparent 70%); border-bottom: 1px solid var(--border); }
  .eyebrow { color: var(--accent-bright); font-size: 0.875rem; font-weight: 600; letter-spacing: 0.12em; text-transform: uppercase; margin-bottom: 14px; }
  .page-head h1 { font-size: clamp(2.2rem, 4vw, 3rem); letter-spacing: -0.025em; margin-bottom: 14px; line-height: 1.1; }
  .page-head h1 em { font-style: italic; font-family: var(--serif); color: var(--accent-bright); font-weight: 400; }
  .lede { color: var(--text-muted); font-size: 1.125rem; max-width: 640px; }
  .featured { padding: 40px 0 20px; }
  .featured-card { display: block; padding: 40px; background: var(--bg-raised); border: 1px solid var(--border-strong); border-radius: var(--radius); color: inherit; text-decoration: none; }
  .featured-card:hover { border-color: var(--accent); }
  .featured-card .tag { display: inline-block; padding: 4px 10px; border-radius: 4px; background: var(--accent-muted); color: var(--accent-bright); font-size: 0.8125rem; font-weight: 600; margin-bottom: 14px; letter-spacing: 0.05em; text-transform: uppercase; }
  .featured-card h2 { font-size: 1.875rem; margin-bottom: 12px; }
  .featured-card p { color: var(--text-muted); font-size: 1.0625rem; margin-bottom: 20px; }
  .featured-meta { display: flex; gap: 18px; color: var(--text-dim); font-size: 0.875rem; }
  .featured-meta strong { color: var(--text); font-weight: 500; }
  .ex-list { padding: 40px 0 80px; }
  .ex-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 20px; }
  .ex-card { display: grid; grid-template-columns: 100px 1fr; gap: 24px; padding: 28px; background: var(--bg-raised); border: 1px solid var(--border); border-radius: var(--radius); color: inherit; text-decoration: none; }
  .ex-card:hover { border-color: var(--accent); transform: translateY(-2px); }
  .ex-card .number { font-family: var(--serif); font-size: 3rem; color: var(--accent-bright); font-style: italic; line-height: 1; }
  .ex-card .tag { display: inline-block; padding: 4px 10px; border-radius: 4px; background: var(--accent-muted); color: var(--accent-bright); font-size: 0.8125rem; font-weight: 600; margin-bottom: 12px; text-transform: uppercase; letter-spacing: 0.05em; }
  .ex-card h3 { font-size: 1.25rem; margin-bottom: 8px; }
  .ex-card p { color: var(--text-muted); font-size: 0.9375rem; margin-bottom: 14px; }
  .ex-card .meta { color: var(--text-dim); font-size: 0.875rem; font-family: var(--mono); }
  @media (max-width: 1024px) { .ex-grid { grid-template-columns: 1fr; } }
</style>
```

- [ ] **Step 2: Build, commit**

```bash
cd site && bun run build && git add site/src/pages/examples/index.astro && \
  git commit -m "feat(examples): index page from content collection"
```

---

### Task 6.3: Single example page

**Files:**
- Create: `site/src/layouts/ArticleLayout.astro`
- Create: `site/src/pages/examples/[...slug].astro`

- [ ] **Step 1: ArticleLayout (shared with blog posts)**

Create `site/src/layouts/ArticleLayout.astro`:

```astro
---
import BaseLayout from './BaseLayout.astro'
interface Props {
  title: string
  description?: string
  eyebrow?: string
  meta?: string
}
const { title, description, eyebrow, meta } = Astro.props
---
<BaseLayout title={title} description={description}>
<main id="main" class="article-main">
  <article class="container article">
    {eyebrow && <p class="art-eyebrow">{eyebrow}</p>}
    <h1>{title}</h1>
    {meta && <p class="art-meta">{meta}</p>}
    <div class="art-body">
      <slot />
    </div>
  </article>
</main>
</BaseLayout>

<style>
  .article-main { padding: 48px 0 80px; }
  .article { max-width: 800px; }
  .art-eyebrow { color: var(--accent-bright); font-size: 0.875rem; font-weight: 600; letter-spacing: 0.12em; text-transform: uppercase; margin-bottom: 14px; }
  .article h1 { font-size: clamp(2rem, 4vw, 2.6rem); letter-spacing: -0.02em; margin-bottom: 12px; line-height: 1.15; }
  .art-meta { color: var(--text-dim); font-size: 0.9375rem; margin-bottom: 32px; font-family: var(--mono); }
  .art-body :global(h2) { font-size: 1.625rem; margin: 48px 0 16px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
  .art-body :global(h3) { font-size: 1.25rem; margin: 32px 0 12px; }
  .art-body :global(p) { margin-bottom: 16px; color: var(--text); line-height: 1.7; }
  .art-body :global(ul), .art-body :global(ol) { margin-bottom: 16px; padding-left: 24px; }
  .art-body :global(li) { margin-bottom: 6px; line-height: 1.7; }
  .art-body :global(a) { color: var(--accent-bright); text-decoration: underline; text-underline-offset: 3px; }
  .art-body :global(code) { font-family: var(--mono); font-size: 0.875em; padding: 2px 6px; background: var(--bg-raised); border: 1px solid var(--border); border-radius: 4px; color: var(--accent-bright); }
  .art-body :global(pre) { background: var(--bg-raised); border: 1px solid var(--border); border-radius: 8px; padding: 16px 18px; overflow-x: auto; margin-bottom: 20px; }
  .art-body :global(pre code) { background: transparent; border: 0; padding: 0; color: var(--text); }
</style>
```

- [ ] **Step 2: Dynamic route**

Create `site/src/pages/examples/[...slug].astro`:

```astro
---
import { getCollection } from 'astro:content'
import ArticleLayout from '../../layouts/ArticleLayout.astro'

export async function getStaticPaths() {
  const entries = await getCollection('examples', e => !e.data.draft)
  return entries.map(entry => ({ params: { slug: entry.slug }, props: { entry } }))
}

const { entry } = Astro.props
const { Content } = await entry.render()
const meta = `${entry.data.steps} steps · ${entry.data.minutes} min · ${entry.data.pillars.join(' · ')}`
---
<ArticleLayout
  title={entry.data.title}
  description={entry.data.description}
  eyebrow={entry.data.tag}
  meta={meta}
>
  <Content />
</ArticleLayout>
```

- [ ] **Step 3: Build, verify, commit**

```bash
cd site && bun run build
git add site/src/layouts/ArticleLayout.astro site/src/pages/examples/[...slug].astro
git commit -m "feat(examples): single-example page using ArticleLayout"
```

**🛑 Checkpoint:** Phase 6 done. Walkthroughs render from MDX.

---

## Phase 7 — Docs migration

**Goal of phase:** every `docs/src/*.md` page lives in `site/src/content/docs/` and renders at `/docs/{...}`.

### Task 7.1: Docs content collection schema

**Files:**
- Modify: `site/src/content/config.ts`

- [ ] **Step 1: Add the schema**

Replace `site/src/content/config.ts` with:

```ts
import { defineCollection, z } from 'astro:content'

const docs = defineCollection({
  type: 'content',
  schema: z.object({
    title: z.string(),
    description: z.string().optional(),
    section: z.enum(['Getting started', 'The six pillars', 'Reference', 'Resources']),
    order: z.number().int().nonnegative(),
  }),
})

const examples = defineCollection({
  type: 'content',
  schema: z.object({
    title: z.string(),
    tag: z.enum(['Rust CLI', 'TS + Bun', 'Python', 'Go', 'Plugins', 'AI', 'CI / CD', 'Monorepo', 'Templates']),
    steps: z.number().int().positive(),
    minutes: z.number().int().positive(),
    pillars: z.array(z.string()),
    description: z.string(),
    featured: z.boolean().default(false),
    draft: z.boolean().default(false),
    order: z.number().optional(),
  }),
})

const blog = defineCollection({
  type: 'content',
  schema: z.object({
    title: z.string(),
    description: z.string(),
    category: z.enum(['announce', 'plugin', 'release', 'workflow', 'tutorial']),
    date: z.date(),
    author: z.string(),
    readTime: z.number().int().positive(),
    featured: z.boolean().default(false),
    draft: z.boolean().default(false),
  }),
})

export const collections = { docs, examples, blog }
```

- [ ] **Step 2: Commit**

```bash
git add site/src/content/config.ts
git commit -m "feat(content): docs and blog collection schemas"
```

---

### Task 7.2: Migrate mdBook content

**Files:**
- Create: many under `site/src/content/docs/`
- Reads from: `docs/src/*.md`

- [ ] **Step 1: Copy and tag each file**

For each file in `docs/src/*.md` and `docs/src/getting-started/*.md`, copy to `site/src/content/docs/` with a frontmatter block prepended. Use the table below for `section` and `order` values.

| Source path | Destination | Section | Order |
|---|---|---|---|
| `docs/src/introduction.md` | `site/src/content/docs/index.md` | Getting started | 0 |
| `docs/src/getting-started/installation.md` | `site/src/content/docs/getting-started/installation.md` | Getting started | 1 |
| `docs/src/getting-started/quick-start.md` | `site/src/content/docs/getting-started/quick-start.md` | Getting started | 2 |
| `docs/src/getting-started/existing-projects.md` | `site/src/content/docs/getting-started/existing-projects.md` | Getting started | 3 |
| `docs/src/pillars.md` | `site/src/content/docs/pillars.md` | The six pillars | 0 |
| `docs/src/templates.md` | `site/src/content/docs/templates.md` | The six pillars | 1 |
| `docs/src/lanes.md` | `site/src/content/docs/lanes.md` | The six pillars | 2 |
| `docs/src/spec.md` | `site/src/content/docs/spec.md` | The six pillars | 3 |
| `docs/src/ask.md` (if exists) / `docs/src/review.md` | `site/src/content/docs/ai.md` | The six pillars | 4 |
| `docs/src/ship.md` | `site/src/content/docs/ship.md` | The six pillars | 5 |
| `docs/src/plugins.md` | `site/src/content/docs/plugins.md` | The six pillars | 6 |
| `docs/src/cli-reference.md` | `site/src/content/docs/reference/cli-reference.md` | Reference | 0 |
| `docs/src/fledge-toml.md` | `site/src/content/docs/reference/fledge-toml.md` | Reference | 1 |
| `docs/src/configuration.md` | `site/src/content/docs/reference/configuration.md` | Reference | 2 |
| `docs/src/doctor.md` | `site/src/content/docs/reference/doctor.md` | Reference | 3 |
| `docs/src/agents.md` | `site/src/content/docs/reference/agents.md` | Reference | 4 |
| `docs/src/template-authoring.md` | `site/src/content/docs/resources/template-authoring.md` | Resources | 0 |
| `docs/src/changelog.md` | `site/src/content/docs/resources/changelog.md` | Resources | 1 |
| `docs/src/github-integration.md` | `site/src/content/docs/resources/github-integration.md` | Resources | 2 |
| `docs/src/reference.md` | `site/src/content/docs/reference/overview.md` | Reference | 99 |

Frontmatter template (replace `<title>` and the table values):

```yaml
---
title: "<title>"
section: "<section>"
order: <order>
---
```

The title can come from the first `# Heading` line of the source file. After adding frontmatter, delete the duplicate `# Heading` line (Astro renders it from `title`).

Concrete one-line per file (shell):

```bash
for src in docs/src/lanes.md; do
  dest=site/src/content/docs/lanes.md
  title=$(grep -m1 '^# ' "$src" | sed 's/^# //')
  { printf -- '---\ntitle: "%s"\nsection: "The six pillars"\norder: 2\n---\n\n' "$title"; \
    awk '/^# /&&!seen{seen=1;next} {print}' "$src"; } > "$dest"
done
```

Adapt the title / section / order per row. Do this for every file in the table.

- [ ] **Step 2: Build to confirm schemas validate**

```bash
cd site && bun run build
```

Expected: build succeeds. If any file errors on schema validation, fix the frontmatter.

- [ ] **Step 3: Commit**

```bash
git add site/src/content/docs/
git commit -m "feat(docs): migrate mdBook content into Astro docs collection"
```

---

### Task 7.3: Sidebar component

**Files:**
- Create: `site/src/components/Sidebar.astro`

- [ ] **Step 1: Implement**

Create `site/src/components/Sidebar.astro`:

```astro
---
import { getCollection } from 'astro:content'
const base = import.meta.env.BASE_URL
const all = await getCollection('docs')
const sectionOrder = ['Getting started', 'The six pillars', 'Reference', 'Resources'] as const
const grouped = sectionOrder.map(name => ({
  name,
  items: all
    .filter(d => d.data.section === name)
    .sort((a, b) => a.data.order - b.data.order),
}))
const current = Astro.url.pathname.replace(/\/$/, '')
const linkFor = (slug: string) => {
  const clean = slug === 'index' ? '' : `/${slug}`
  return `${base}docs${clean}`.replace(/\/+/g, '/')
}
---
<aside class="sidebar" aria-label="Documentation navigation">
  {grouped.map(group => (
    <>
      <h2 class="section-title">{group.name}</h2>
      <ul>
        {group.items.map(item => {
          const href = linkFor(item.slug)
          const active = href.replace(/\/$/, '') === current
          return <li><a href={href} class={active ? 'active' : ''} aria-current={active ? 'page' : undefined}>{item.data.title}</a></li>
        })}
      </ul>
    </>
  ))}
</aside>

<style is:global>
  .sidebar { padding: 32px 24px; border-right: 1px solid var(--border); }
  .sidebar h2.section-title { color: var(--text-dim); font-size: 0.75rem;
    font-weight: 600; letter-spacing: 0.12em; text-transform: uppercase;
    margin-bottom: 10px; margin-top: 24px; }
  .sidebar h2.section-title:first-of-type { margin-top: 0; }
  .sidebar ul { padding: 0; }
  .sidebar li { margin-bottom: 2px; }
  .sidebar a { display: block; padding: 7px 12px; color: var(--text-muted);
    font-size: 0.9375rem; border-radius: 6px; border-left: 2px solid transparent;
    line-height: 1.45; }
  .sidebar a:hover { color: var(--text); background: var(--bg-raised); }
  .sidebar a.active { color: var(--accent-bright); background: var(--accent-muted);
    border-left-color: var(--accent); }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add site/src/components/Sidebar.astro
git commit -m "feat(docs): Sidebar component built from docs collection"
```

---

### Task 7.4: DocsLayout (sidebar + article)

**Files:**
- Create: `site/src/layouts/DocsLayout.astro`

- [ ] **Step 1: Implement**

Create `site/src/layouts/DocsLayout.astro`:

```astro
---
import BaseLayout from './BaseLayout.astro'
import Sidebar from '../components/Sidebar.astro'
interface Props {
  title: string
  description?: string
  section: string
}
const { title, description, section } = Astro.props
const base = import.meta.env.BASE_URL
---
<BaseLayout title={`${title} — fledge docs`} description={description}>
<div class="docs-grid">
  <Sidebar />
  <main id="main" class="article">
    <nav class="breadcrumb" aria-label="Breadcrumb">
      <a href={`${base}docs`}>Docs</a>
      <span class="sep" aria-hidden="true">/</span>
      <span>{section}</span>
      <span class="sep" aria-hidden="true">/</span>
      <span>{title}</span>
    </nav>
    <h1>{title}</h1>
    <div class="doc-body">
      <slot />
    </div>
  </main>
</div>
</BaseLayout>

<style>
  .docs-grid { display: grid; grid-template-columns: 260px 1fr; max-width: 1400px; margin: 0 auto; }
  .article { padding: 48px 56px; max-width: 800px; }
  .breadcrumb { display: flex; gap: 8px; align-items: center; color: var(--text-dim); font-size: 0.875rem; margin-bottom: 20px; flex-wrap: wrap; }
  .breadcrumb a { color: var(--text-muted); }
  .breadcrumb a:hover { color: var(--accent-bright); }
  .breadcrumb .sep { color: var(--border-strong); }
  .article h1 { font-size: clamp(2rem, 4vw, 2.6rem); letter-spacing: -0.02em; margin-bottom: 12px; line-height: 1.15; }
  .doc-body :global(h2) { font-size: 1.625rem; margin: 48px 0 16px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
  .doc-body :global(h3) { font-size: 1.25rem; margin: 32px 0 12px; }
  .doc-body :global(p) { margin-bottom: 16px; line-height: 1.7; }
  .doc-body :global(ul), .doc-body :global(ol) { margin-bottom: 16px; padding-left: 24px; }
  .doc-body :global(li) { margin-bottom: 6px; line-height: 1.7; }
  .doc-body :global(a) { color: var(--accent-bright); text-decoration: underline; text-underline-offset: 3px; }
  .doc-body :global(code) { font-family: var(--mono); font-size: 0.875em; padding: 2px 6px; background: var(--bg-raised); border: 1px solid var(--border); border-radius: 4px; color: var(--accent-bright); }
  .doc-body :global(pre) { background: var(--bg-raised); border: 1px solid var(--border); border-radius: 8px; padding: 16px 18px; overflow-x: auto; margin-bottom: 20px; }
  .doc-body :global(pre code) { background: transparent; border: 0; padding: 0; color: var(--text); }
  @media (max-width: 900px) {
    .docs-grid { grid-template-columns: 1fr; }
    .sidebar { display: none; }
    .article { padding: 28px 20px; }
  }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add site/src/layouts/DocsLayout.astro
git commit -m "feat(docs): DocsLayout with sidebar and article column"
```

---

### Task 7.5: Docs index + dynamic route

**Files:**
- Create: `site/src/pages/docs/index.astro`
- Create: `site/src/pages/docs/[...slug].astro`

- [ ] **Step 1: Index**

Create `site/src/pages/docs/index.astro`:

```astro
---
import { getEntry } from 'astro:content'
import DocsLayout from '../../layouts/DocsLayout.astro'
const entry = await getEntry('docs', 'index')
const { Content } = await entry.render()
---
<DocsLayout title={entry.data.title} description={entry.data.description} section={entry.data.section}>
  <Content />
</DocsLayout>
```

- [ ] **Step 2: Dynamic page**

Create `site/src/pages/docs/[...slug].astro`:

```astro
---
import { getCollection } from 'astro:content'
import DocsLayout from '../../layouts/DocsLayout.astro'

export async function getStaticPaths() {
  const entries = await getCollection('docs')
  return entries
    .filter(e => e.slug !== 'index')
    .map(entry => ({ params: { slug: entry.slug }, props: { entry } }))
}

const { entry } = Astro.props
const { Content } = await entry.render()
---
<DocsLayout title={entry.data.title} description={entry.data.description} section={entry.data.section}>
  <Content />
</DocsLayout>
```

- [ ] **Step 3: Build and verify**

```bash
cd site && bun run build
```

Visit a few docs URLs in `site/dist/docs/lanes/index.html` etc.

- [ ] **Step 4: Commit**

```bash
git add site/src/pages/docs/index.astro site/src/pages/docs/[...slug].astro
git commit -m "feat(docs): index + dynamic docs route"
```

**🛑 Checkpoint:** Phase 7 done. All mdBook content live at `/docs/...`.

---

## Phase 8 — Blog

**Goal of phase:** `/blog` lists posts; `/blog/{slug}` renders each one.

### Task 8.1: Seed posts

**Files:**
- Create: `site/src/content/blog/welcome.mdx`
- Create: `site/src/content/blog/v1-4-1-release.mdx`

- [ ] **Step 1: Welcome post**

Create `site/src/content/blog/welcome.mdx`:

```mdx
---
title: "Marketing site rebuild — what's coming and why"
description: "Why we're moving off mdBook for the home page (and keeping it for the docs), what the new plugin registry looks like, what's in the v1.5 milestone."
category: announce
date: 2026-05-17
author: Leif
readTime: 6
featured: true
---

(Post body — placeholder. Replace with real announcement before launch.)
```

- [ ] **Step 2: Release post**

Create `site/src/content/blog/v1-4-1-release.mdx`:

```mdx
---
title: "fledge v1.4.1 — six pillars, thirty-one plugins, one binary"
description: "The point-one ships seven bug fixes from the v1.4 release, polishes spec-check error messages, and lays groundwork for the upcoming plugin registry rebuild."
category: release
date: 2026-05-11
author: Leif
readTime: 4
---

(Release notes body — placeholder. Pull from CHANGELOG before launch.)
```

- [ ] **Step 3: Commit**

```bash
git add site/src/content/blog/
git commit -m "feat(blog): two seed posts (welcome announcement + v1.4.1 release)"
```

---

### Task 8.2: PostCard + blog index

**Files:**
- Create: `site/src/components/PostCard.astro`
- Create: `site/src/pages/blog/index.astro`

- [ ] **Step 1: PostCard**

Create `site/src/components/PostCard.astro`:

```astro
---
import CategoryTag from './CategoryTag.astro'
import type { CollectionEntry } from 'astro:content'
interface Props { post: CollectionEntry<'blog'> }
const { post } = Astro.props
const base = import.meta.env.BASE_URL
const date = post.data.date.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' })
const initial = post.data.author.charAt(0)
---
<li><a href={`${base}blog/${post.slug}`} class="post">
  <div class="post-top">
    <CategoryTag category={post.data.category} />
    <span class="post-date">{date}</span>
  </div>
  <h3>{post.data.title}</h3>
  <p>{post.data.description}</p>
  <div class="post-foot">
    <span class="avatar-sm" aria-hidden="true">{initial}</span> {post.data.author}
    <span class="read">{post.data.readTime} min read</span>
  </div>
</a></li>

<style is:global>
  .post { display: flex; flex-direction: column; gap: 14px;
    padding: 24px; background: var(--bg-raised);
    border: 1px solid var(--border); border-radius: var(--radius);
    color: inherit; text-decoration: none; transition: all .15s; }
  .post:hover { border-color: var(--accent); transform: translateY(-2px); }
  .post-top { display: flex; justify-content: space-between; gap: 10px; align-items: center; }
  .post-date { color: var(--text-dim); font-size: 0.8125rem; font-family: var(--mono); }
  .post h3 { font-size: 1.125rem; letter-spacing: -0.005em; line-height: 1.3; flex: 1; }
  .post p { color: var(--text-muted); font-size: 0.9375rem; line-height: 1.55; flex: 1; }
  .post-foot { display: flex; align-items: center; gap: 10px;
    padding-top: 12px; border-top: 1px dashed var(--border-strong);
    color: var(--text-dim); font-size: 0.8125rem; }
  .post-foot .avatar-sm { width: 24px; height: 24px; border-radius: 50%;
    background: linear-gradient(135deg, var(--accent-bright), var(--accent));
    display: inline-grid; place-items: center; color: #1a0f08;
    font-size: 0.7rem; font-weight: 700; }
  .post-foot .read { margin-left: auto; }
</style>
```

- [ ] **Step 2: Blog index**

Create `site/src/pages/blog/index.astro`:

```astro
---
import { getCollection } from 'astro:content'
import BaseLayout from '../../layouts/BaseLayout.astro'
import PostCard from '../../components/PostCard.astro'
import CategoryTag from '../../components/CategoryTag.astro'
const base = import.meta.env.BASE_URL
const posts = (await getCollection('blog', p => !p.data.draft))
  .sort((a, b) => +b.data.date - +a.data.date)
const featured = posts.find(p => p.data.featured)
const rest = posts.filter(p => p !== featured)
const fmt = (d: Date) => d.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' })
---
<BaseLayout title="fledge blog">
<main id="main">

<section class="page-head">
  <div class="container head-row">
    <div>
      <p class="eyebrow">The fledge blog</p>
      <h1>Updates, plugins, and <em>field notes.</em></h1>
      <p class="lede">Release notes, plugin spotlights, workflow deep-dives, and the occasional design rant.</p>
    </div>
    <div class="rss-row">
      <a href={`${base}rss.xml`} class="rss"><span aria-hidden="true">⌗</span> RSS</a>
    </div>
  </div>
</section>

{featured && (
  <section class="featured">
    <div class="container">
      <a href={`${base}blog/${featured.slug}`} class="feat-card">
        <div class="feat-meta">
          <CategoryTag category={featured.data.category} />
          <span class="feat-date">{fmt(featured.data.date)} · {featured.data.readTime} min read</span>
        </div>
        <h2>{featured.data.title}</h2>
        <p class="dek">{featured.data.description}</p>
        <p class="byline">by {featured.data.author}</p>
      </a>
    </div>
  </section>
)}

<section class="post-list">
  <div class="container">
    <div class="list-head">
      <h2>All posts</h2>
      <span class="count">{posts.length} posts</span>
    </div>
    <ul class="post-grid">
      {rest.map(p => <PostCard post={p} />)}
    </ul>
  </div>
</section>

<section class="read-more">
  <div class="container">
    <a href={`${base}plugins`} class="rm-card">Browse 31 plugins <span aria-hidden="true">→</span></a>
    <a href={`${base}docs`} class="rm-card">Read the docs <span aria-hidden="true">→</span></a>
    <a href="https://github.com/CorvidLabs/fledge" class="rm-card">Star on GitHub <span aria-hidden="true">↗</span></a>
  </div>
</section>

</main>
</BaseLayout>

<style>
  .page-head { padding: 60px 0 36px; background-image: radial-gradient(ellipse 1200px 400px at 50% -100px, rgba(234,88,12,0.10), transparent 70%); border-bottom: 1px solid var(--border); }
  .head-row { display: flex; align-items: end; justify-content: space-between; gap: 24px; flex-wrap: wrap; }
  .eyebrow { color: var(--accent-bright); font-size: 0.875rem; font-weight: 600; letter-spacing: 0.12em; text-transform: uppercase; margin-bottom: 14px; }
  .page-head h1 { font-size: clamp(2.2rem, 4vw, 3rem); letter-spacing: -0.025em; line-height: 1.1; margin-bottom: 10px; }
  .page-head h1 em { font-style: italic; font-family: var(--serif); color: var(--accent-bright); font-weight: 400; }
  .lede { color: var(--text-muted); font-size: 1.125rem; max-width: 580px; }
  .rss { padding: 10px 14px; background: var(--bg-raised); border: 1px solid var(--border-strong); border-radius: 8px; color: var(--text-muted); font-size: 0.875rem; display: inline-flex; gap: 6px; align-items: center; }
  .rss:hover { color: var(--accent-bright); border-color: var(--accent); }
  .featured { padding: 40px 0 20px; }
  .feat-card { display: block; padding: 40px; background: var(--bg-raised); border: 1px solid var(--border-strong); border-radius: var(--radius); color: inherit; text-decoration: none; }
  .feat-card:hover { border-color: var(--accent); }
  .feat-meta { display: flex; gap: 12px; align-items: center; margin-bottom: 16px; flex-wrap: wrap; }
  .feat-date { color: var(--text-dim); font-size: 0.875rem; font-family: var(--mono); }
  .feat-card h2 { font-size: clamp(1.6rem, 3vw, 2.1rem); line-height: 1.15; margin-bottom: 14px; }
  .feat-card .dek { color: var(--text-muted); font-size: 1.0625rem; line-height: 1.55; margin-bottom: 12px; }
  .feat-card .byline { color: var(--text-dim); font-size: 0.9375rem; }
  .post-list { padding: 40px 0 60px; }
  .list-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
  .list-head h2 { font-size: 1.5rem; }
  .list-head .count { color: var(--text-dim); font-size: 0.9375rem; }
  .post-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; }
  .read-more { padding: 40px 0 80px; }
  .read-more .container { display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; }
  .rm-card { display: block; padding: 22px; background: var(--bg-raised); border: 1px solid var(--border); border-radius: var(--radius); color: var(--text); font-weight: 500; text-decoration: none; transition: border-color .15s; }
  .rm-card:hover { border-color: var(--accent); color: var(--accent-bright); }
  @media (max-width: 1024px) { .post-grid { grid-template-columns: repeat(2, 1fr); } }
  @media (max-width: 760px) { .post-grid, .read-more .container { grid-template-columns: 1fr; } }
</style>
```

- [ ] **Step 3: Build, commit**

```bash
cd site && bun run build
git add site/src/components/PostCard.astro site/src/pages/blog/index.astro
git commit -m "feat(blog): index page with featured post + post grid + read-more strip"
```

---

### Task 8.3: Single blog post page

**Files:**
- Create: `site/src/pages/blog/[...slug].astro`

- [ ] **Step 1: Implement**

Create `site/src/pages/blog/[...slug].astro`:

```astro
---
import { getCollection } from 'astro:content'
import ArticleLayout from '../../layouts/ArticleLayout.astro'

export async function getStaticPaths() {
  const entries = await getCollection('blog', p => !p.data.draft)
  return entries.map(entry => ({ params: { slug: entry.slug }, props: { entry } }))
}

const { entry } = Astro.props
const { Content } = await entry.render()
const date = entry.data.date.toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })
const meta = `${date} · ${entry.data.author} · ${entry.data.readTime} min read`
---
<ArticleLayout
  title={entry.data.title}
  description={entry.data.description}
  eyebrow={entry.data.category}
  meta={meta}
>
  <Content />
</ArticleLayout>
```

- [ ] **Step 2: Build, commit**

```bash
cd site && bun run build
git add site/src/pages/blog/[...slug].astro
git commit -m "feat(blog): single-post page using ArticleLayout"
```

**🛑 Checkpoint:** Phase 8 done.

---

## Phase 9 — Build & deploy + cutover

**Goal of phase:** old mdBook tree is deleted; new pages.yml ships the Astro build on every push and weekly cron.

### Task 9.1: Doc redirects generator

**Files:**
- Create: `site/scripts/generate-doc-redirects.ts`
- Create: `site/scripts/generate-doc-redirects.test.ts`

mdBook output URL form: `corvidlabs.github.io/fledge/lanes.html`. New form: `corvidlabs.github.io/fledge/docs/lanes`. We generate one static HTML file per old URL that does an HTTP-equiv refresh + JS replace.

- [ ] **Step 1: Failing test**

Create `site/scripts/generate-doc-redirects.test.ts`:

```ts
import { describe, test, expect } from 'bun:test'
import { redirectHtml, computeRedirects } from './generate-doc-redirects'

describe('redirectHtml', () => {
  test('emits meta-refresh + canonical', () => {
    const html = redirectHtml('/fledge/docs/lanes')
    expect(html).toContain('<meta http-equiv="refresh" content="0; url=/fledge/docs/lanes">')
    expect(html).toContain('canonical')
    expect(html).toContain('location.replace')
  })
})

describe('computeRedirects', () => {
  test('maps top-level mdBook pages to new docs paths', () => {
    const mapped = computeRedirects(['lanes.md', 'pillars.md', 'getting-started/installation.md'])
    expect(mapped['lanes.html']).toBe('/fledge/docs/lanes')
    expect(mapped['pillars.html']).toBe('/fledge/docs/pillars')
    expect(mapped['getting-started/installation.html']).toBe('/fledge/docs/getting-started/installation')
  })
})
```

- [ ] **Step 2: Implement**

Create `site/scripts/generate-doc-redirects.ts`:

```ts
import { readdirSync, statSync, mkdirSync, writeFileSync } from 'node:fs'
import { join, dirname, relative } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const PUBLIC_DIR = join(__dirname, '..', 'public')
const BASE = '/fledge/'

export function redirectHtml(target: string): string {
  return `<!DOCTYPE html>
<html><head>
<meta charset="utf-8">
<title>Redirecting…</title>
<link rel="canonical" href="${target}">
<meta http-equiv="refresh" content="0; url=${target}">
<script>location.replace(${JSON.stringify(target)})</script>
</head><body>
<p>This page has moved. <a href="${target}">Continue →</a></p>
</body></html>
`
}

export function computeRedirects(mdFiles: string[]): Record<string, string> {
  const out: Record<string, string> = {}
  for (const f of mdFiles) {
    if (f === 'SUMMARY.md') continue
    const html = f.replace(/\.md$/, '.html')
    const newPath = `${BASE}docs/${f.replace(/\.md$/, '')}`.replace(/\/+/g, '/')
    out[html] = newPath
  }
  return out
}

function walk(dir: string, prefix = ''): string[] {
  const out: string[] = []
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry)
    const rel = prefix ? `${prefix}/${entry}` : entry
    if (statSync(full).isDirectory()) out.push(...walk(full, rel))
    else if (entry.endsWith('.md')) out.push(rel)
  }
  return out
}

function main() {
  // Source-of-truth = the migrated docs/ tree under site/src/content/docs
  const docsSrc = join(__dirname, '..', 'src', 'content', 'docs')
  const mdFiles = walk(docsSrc)
  const mapped = computeRedirects(mdFiles)
  for (const [oldPath, newPath] of Object.entries(mapped)) {
    const dest = join(PUBLIC_DIR, oldPath)
    mkdirSync(dirname(dest), { recursive: true })
    writeFileSync(dest, redirectHtml(newPath))
  }
  console.log(`[generate-doc-redirects] wrote ${Object.keys(mapped).length} redirect files`)
}

if (import.meta.main) main()
```

- [ ] **Step 3: Run tests**

```bash
cd site && bun test scripts/generate-doc-redirects.test.ts
```

Expected: green.

- [ ] **Step 4: Re-enable in package.json**

Change the `prebuild` line in `site/package.json` to:

```json
"prebuild": "bun scripts/build-plugin-registry.ts && bun scripts/generate-doc-redirects.ts",
```

- [ ] **Step 5: Build, verify, commit**

```bash
cd site && GITHUB_TOKEN=$(gh auth token) bun run build
ls site/dist/lanes.html site/dist/pillars.html  # confirm redirects exist
git add site/scripts/generate-doc-redirects.ts site/scripts/generate-doc-redirects.test.ts site/package.json
git commit -m "feat(site/scripts): generate redirect HTML files for old mdBook URLs"
```

---

### Task 9.2: GitHub Actions workflow

**Files:**
- Create: `.github/workflows/pages.yml`

- [ ] **Step 1: Add the workflow**

Create `.github/workflows/pages.yml`:

```yaml
name: Deploy Site

on:
  push:
    branches: [main]
    paths:
      - 'site/**'
      - '.github/workflows/pages.yml'
  workflow_dispatch:
  schedule:
    # Weekly Monday 08:00 UTC — refreshes plugin registry
    - cron: '0 8 * * 1'

concurrency:
  group: pages
  cancel-in-progress: true

permissions:
  contents: read
  pages: write
  id-token: write

jobs:
  build:
    name: Build Astro Site
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Bun
        uses: oven-sh/setup-bun@v2

      - name: Restore plugin registry cache
        uses: actions/cache@v4
        with:
          path: |
            site/src/data/plugins.json
            site/src/data/plugins
          key: plugin-registry-${{ github.run_id }}
          restore-keys: |
            plugin-registry-

      - name: Install dependencies
        working-directory: site
        run: bun install --frozen-lockfile

      - name: Run unit tests
        working-directory: site
        run: bun test

      - name: Build site (runs prebuild for plugin registry + redirects)
        working-directory: site
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: bun run build

      - name: Upload Pages artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: site/dist

  deploy:
    name: Deploy to GitHub Pages
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - id: deployment
        uses: actions/deploy-pages@v4
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/pages.yml
git commit -m "ci(pages): deploy Astro site on push, manual trigger, and weekly cron"
```

---

### Task 9.3: Cutover — delete mdBook tree and old workflow

**Files:**
- Delete: `docs/src/` (all of it)
- Delete: `docs/book.toml`
- Delete: `docs/book/` (gitignored anyway, but remove from working tree)
- Delete: `.github/workflows/docs.yml`
- Modify: `.gitignore` (remove `docs/book/` line, no longer relevant)
- Modify: `README.md` (update docs badge if it references the old URL — same URL though, so likely no change)

- [ ] **Step 1: Run the deletions**

```bash
git rm -r docs/src docs/book.toml
rm -rf docs/book
git rm .github/workflows/docs.yml
```

- [ ] **Step 2: Tidy .gitignore**

In `.gitignore`, remove the line `docs/book/`.

- [ ] **Step 3: Build, verify a few URLs**

```bash
cd site && GITHUB_TOKEN=$(gh auth token) bun run build
# spot-check
test -f dist/index.html
test -f dist/plugins/index.html
test -f dist/docs/index.html
test -f dist/lanes.html  # redirect file
test -f dist/404.html
```

- [ ] **Step 4: Commit the cutover**

```bash
git add .gitignore
git commit -m "chore(docs): delete mdBook tree and docs.yml; cut over to Astro site"
```

---

### Task 9.4: Push branch, open PR, smoke-test deploy

- [ ] **Step 1: Push the branch**

```bash
git push -u origin docs/marketing-site-spec
```

- [ ] **Step 2: Open the PR**

```bash
gh pr create --title "Marketing site rebuild — Astro + plugin registry" --body "$(cat <<'EOF'
## Summary
- Replaces the mdBook GitHub Pages site with an Astro + MDX marketing site.
- Adds a first-class plugin registry (`/plugins` + `/plugins/{slug}`) backed by a build-time GitHub fetch.
- Migrates all existing mdBook docs into the new site under `/docs`.
- Generates 1-step redirects from old top-level URLs (`/fledge/lanes.html` etc.) to the new docs paths.
- Replaces `.github/workflows/docs.yml` with `pages.yml` (push + manual + weekly cron).

Spec: `docs/superpowers/specs/2026-05-17-marketing-site-design.md`
Plan: `docs/superpowers/plans/2026-05-17-marketing-site.md`

## Test plan
- [ ] CI runs all `bun test` suites and they pass
- [ ] `bun run build` produces `site/dist/` with `/`, `/plugins`, `/plugins/{slug}`, `/examples`, `/docs/...`, `/blog`, `/blog/{slug}`, `404`
- [ ] Old mdBook URLs return either content or a 1-step redirect (verify `/fledge/lanes.html` lands on `/fledge/docs/lanes`)
- [ ] `prefers-reduced-motion` produces a no-motion page
- [ ] Lighthouse ≥ 95 on Performance, Accessibility, Best Practices, SEO for `/`, `/plugins`, a sample `/docs/{page}`
- [ ] axe-core scan on `/`, `/plugins`, `/docs/lanes`, `/blog` produces zero violations

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 3: Trigger a deploy manually after merge**

After the PR merges, kick the workflow once:

```bash
gh workflow run pages.yml
```

Watch it:

```bash
gh run watch
```

Once green, visit `https://corvidlabs.github.io/fledge/` and verify the new site is live.

**🛑 Checkpoint:** Phase 9 done. Site is live.

---

## Phase 10 — A11y + Lighthouse polish

**Goal of phase:** hit the success criteria from the spec.

### Task 10.1: axe-core scan

**Files:**
- Create: `site/scripts/run-axe.ts` (optional one-shot helper)

- [ ] **Step 1: Run axe via Playwright (one-shot, manual)**

Install Playwright + axe-core into a scratch dir (not the site project, to keep deps lean):

```bash
mkdir -p /tmp/fledge-axe && cd /tmp/fledge-axe
bun init -y
bun add -d @playwright/test @axe-core/playwright
bunx playwright install chromium
```

Create `/tmp/fledge-axe/scan.ts`:

```ts
import { chromium } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'

const URLS = [
  'http://localhost:4321/fledge/',
  'http://localhost:4321/fledge/plugins',
  'http://localhost:4321/fledge/docs',
  'http://localhost:4321/fledge/docs/lanes',
  'http://localhost:4321/fledge/blog',
]

const browser = await chromium.launch()
const page = await browser.newPage()
for (const url of URLS) {
  await page.goto(url)
  const { violations } = await new AxeBuilder({ page }).analyze()
  console.log(`\n=== ${url} ===`)
  if (violations.length === 0) console.log('✓ no violations')
  for (const v of violations) console.log(`✗ ${v.id} (${v.nodes.length} nodes): ${v.help}`)
}
await browser.close()
```

- [ ] **Step 2: Run the scan**

Terminal A:

```bash
cd ~/fledge/site && bun run preview
```

Terminal B:

```bash
cd /tmp/fledge-axe && bun run scan.ts
```

- [ ] **Step 3: Fix anything reported**

Common axe violations and their fixes:
- `color-contrast` — bump the offending color in `globals.css`.
- `image-alt` — every `<img>` needs `alt=""` (decorative) or descriptive text.
- `landmark-unique` — make sure only one `<main>` exists per page.
- `link-name` — links with only icons need `aria-label`.

For each fix, commit separately with a message describing the violation fixed.

- [ ] **Step 4: Re-run scan, confirm zero violations**

---

### Task 10.2: Lighthouse pass

- [ ] **Step 1: Run Lighthouse on the live deploy**

```bash
bunx lighthouse https://corvidlabs.github.io/fledge/ --view --preset=desktop --output=html --output-path=/tmp/lh-home.html
bunx lighthouse https://corvidlabs.github.io/fledge/plugins --view --preset=desktop --output=html --output-path=/tmp/lh-plugins.html
bunx lighthouse https://corvidlabs.github.io/fledge/docs/lanes --view --preset=desktop --output=html --output-path=/tmp/lh-docs.html
```

- [ ] **Step 2: Address any < 95 scores**

Typical fixes:
- **Performance**: add `width` / `height` to images, preload fonts (or use system fonts only — we already do), ensure no render-blocking JS (Astro is already SSG so this is mostly free).
- **Accessibility**: same fixes as axe.
- **Best Practices**: serve favicon, set CSP meta if needed (not required for v1).
- **SEO**: every page has `<title>` and `<meta name="description">` (already set in BaseLayout).

- [ ] **Step 3: Repeat with `--preset=mobile`** and fix any mobile-only regressions.

- [ ] **Step 4: Commit any tweaks**

```bash
git add site/...
git commit -m "perf/a11y: lighthouse polish — <specific fix>"
```

**🛑 Final checkpoint:** every success criterion from the spec is met. Site is live, registry refreshes weekly, all four nav pages work, per-plugin pages render, old URLs don't 404, motion-reduced users get a static page.

---

## Self-review (run before declaring the plan done)

- **Spec coverage:** every section in the spec maps to a task — Phase 1 covers Scaffold; Phase 2 covers Component vocabulary + Visual identity (palette, type); Phase 3 covers `/`; Phases 4–5 cover `/plugins` + `/plugins/{slug}` + the data pipeline; Phase 6 covers `/examples`; Phase 7 covers `/docs`; Phase 8 covers `/blog`; Phase 9 covers Build & deploy + cutover; Phase 10 covers Success criteria (Lighthouse, a11y, reduced-motion).

- **No-placeholder check:** every step includes exact file paths, exact code, and exact commands. The two seed walkthroughs and seed blog posts say "placeholder content acceptable for v1" — that's documented, not a placeholder in the plan itself.

- **Type consistency:** `RegistryEntry` and `FullEntry` are defined in `build-plugin-registry.ts` and imported by `PluginCard.astro` and `plugins/[slug].astro`. `slugFromName`, `inferLanguage`, `inferTrustTier`, `relatedSlugs`, `renderReadme`, `repoToEntry` names are consistent across the task definitions, the source, and the tests.

- **Order risk:** Task 3.4 introduces a hardcoded plugin spotlight before Phase 4 builds `plugins.json` — flagged in the task itself, fixed when Phase 4 lands (re-wire to read from JSON in a follow-up commit during Task 4.6 or 5.1).
