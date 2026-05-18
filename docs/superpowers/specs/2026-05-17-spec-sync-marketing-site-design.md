# spec-sync marketing site — design spec

**Status:** approved, ready for plan
**Author:** brainstormed with @0xLeif via Claude on 2026-05-17 (sibling spec to the fledge marketing-site rebuild)
**Sibling spec:** `docs/superpowers/specs/2026-05-17-marketing-site-design.md` (fledge — same parent decisions, different content)
**Repo:** `CorvidLabs/spec-sync`
**Replaces:** existing mdBook site at `https://corvidlabs.github.io/spec-sync/` and its `.github/workflows/docs.yml`

## Goal

Replace the mdBook-only `corvidlabs.github.io/spec-sync` site with an Astro + MDX marketing site that mirrors the fledge site's architecture and component vocabulary — but with a sky-blue theme and a **Languages registry** in the slot where fledge has its Plugin registry.

The site does four jobs:

1. **Convert** first-time visitors into installers (hero, CTA, install snippet for crates.io / GitHub Action).
2. **Showcase** the 12 supported languages — what spec-sync detects in each and how detection works.
3. **Educate** with end-to-end walkthroughs and the migrated docs.
4. **Communicate** ongoing work (blog: releases, language additions, integration posts, announcements).

## What's identical to the fledge spec

The following sections are intentionally the same as the fledge spec — implementation should reuse the same patterns and (where possible) the same Astro components, only swapping CSS custom properties:

- Stack: Astro 5, `@astrojs/mdx`, `@astrojs/sitemap`, Bun runtime + test runner, TypeScript.
- Component vocabulary: `Header / Footer / Button / Badge / Terminal / Pillar / CategoryTag / Callout / Sidebar / PostCard / ArticleLayout / DocsLayout / BaseLayout`. **Same component names, same props, same structural CSS.**
- Accessibility baseline: body ≥ 16px, secondary ≥ 14px, focus rings, semantic landmarks, `prefers-reduced-motion`, skip link, color contrast ≥ 4.5:1 for text.
- Blog category color codes (Release/Plugin/Workflow/Tutorial/Announcement) — same color palette, same `CategoryTag` component.
- Docs migration approach: mdBook content moves to an Astro content collection with frontmatter.
- Build & deploy: same `.github/workflows/pages.yml` skeleton, same weekly cron for the registry refresh, same `actions/upload-pages-artifact` flow.
- No mailing list. Blog ends with a "Read more" CTA row (Languages / Docs / GitHub).
- Docs search deferred to v1.1 (browser-native `Cmd/Ctrl+F` only).
- Per-{entity} pages shipped in v1 (per-language pages here, analogous to per-plugin pages there).
- Mailing-list / per-{entity} / docs-search / fetch-auth open questions all resolved the same way as the fledge spec.

The rest of this doc only covers what's **different** from the fledge spec.

## Non-goals

- Custom domain. Stays on `corvidlabs.github.io/spec-sync/`. Astro `base: '/spec-sync/'`.
- Multi-version docs.
- Auto-extracting language metadata from the spec-sync Rust source — hand-maintained `languages.json` is fine for v1.
- Server-rendered search.
- Preserving the existing GitHub Pages analytics setup. (New redirect rules for old top-level docs URLs are added — see Success criteria — but the existing analytics integration is not migrated.)

## Information architecture

```
/                       index.astro             — Landing page
/languages              languages/index.astro   — Language registry (search, filter, list)
/languages/{slug}       languages/[slug].astro  — Per-language page (detection rules, examples, deeper docs)
/examples               examples/index.astro    — Walkthrough index
/examples/{slug}        examples/[...slug].astro — Single walkthrough
/docs                   docs/index.astro
/docs/{section}/{page}  docs/[...slug].astro    — Migrated mdBook content
/blog                   blog/index.astro
/blog/{slug}            blog/[...slug].astro
/404                    404.astro
```

Top nav: **Languages · Examples · Docs · Blog** plus primary `Install` CTA and GitHub link.

## Visual identity

**Family resemblance with fledge — same structural rhythm, different temperature.**

Identical to fledge:
- Dark background. Sticky blurred nav. Italic-serif accents inside sans body. Split-grid hero. 2×3 unified pillars grid with hairline dividers. Stat row. Terminal block. Color-coded blog tags. Footer pattern.
- Type stack (system sans + Iowan/Apple Garamond/Baskerville/Georgia serif + JetBrains/SF Mono).

Different from fledge:
- **Palette** — cool sky-blue instead of warm rust. Background is also cooler.
  - `--bg: #0a1018` (slate-950 with a slight blue tilt)
  - `--bg-raised: #0e1825`
  - `--border: #1c2a3a`
  - `--border-strong: #2a3b4f`
  - `--text: #f1f5f9` (slate-100)
  - `--text-muted: #b8c5d4` (raised for a11y — ≥ 11:1 contrast on bg)
  - `--text-dim: #94a3b8` (slate-400, ≥ 6:1 contrast)
  - `--accent: #0ea5e9` (sky-500)
  - `--accent-bright: #7dd3fc` (sky-300, gradient highlight + focus ring)
  - `--accent-deep: #075985` (sky-800)
  - `--accent-muted: rgba(14, 165, 233, 0.10)`
  - `--accent-glow: rgba(14, 165, 233, 0.18)`
- **Italic-serif accent words** swap: `sync`, `truth`, `honest`, `everywhere` (vs fledge's `fly`, `nothing`, `shipped`, `real`).
- **Logo** — `s` glyph in a 28×28 sky-gradient tile instead of `f` orange-gradient tile. Otherwise identical wordmark pattern.

## Page-by-page content (only the differences from fledge)

### `/` — landing

Same structure as fledge home. Different copy.

1. **Sticky nav**: logo + version chip ("v4.1" or whatever current is — pull from `specsync` crate at build time), primary nav, `Install` CTA.
2. **Hero (split grid)**:
   - Badge: `12 languages · GitHub Marketplace`.
   - H1: "Keep your docs <em>honest.</em>" (italic-serif on "honest")
   - Subtitle: "Bidirectional spec-to-code validation. Cross-project refs. AI generation. CI-enforced contract checking — in any language."
   - CTAs: `Get started →` (primary, → `/docs/getting-started`), `Browse 12 languages` (secondary, → `/languages`).
   - Fine-print: `cargo install specsync` (with copy affordance).
   - Right column: terminal block showing `spec-sync check` finding a phantom export, then explaining it.
3. **Stats row**: `12 languages · 6 powers · CI-ready · 1 binary`.
4. **Six powers** (the equivalent of fledge's Six Pillars). 2×3 grid:
   - **i. Validate** — `$ spec-sync check` — Code → spec and spec → code. Errors on drift.
   - **ii. Languages** — 12 detected — TypeScript, Rust, Go, Python, Swift, Kotlin, Java, C#, Dart, PHP, Ruby, YAML. Same spec format for all.
   - **iii. Generate** — `$ spec-sync generate` — AI-assisted spec generation from existing code.
   - **iv. Integrate** — VS Code extension + GitHub Action + CLI. Drop into any CI.
   - **v. Cross-refs** — Reference exports across repos. Dependency graph included.
   - **vi. Extend** — Custom validators, hooks, output formats. Open file formats.
5. **Languages spotlight** — 4 language cards (same layout as fledge's plugin spotlight) showing TS / Rust / Go / Python with a small example line per language.
6. **Examples teaser** — 3 cards (e.g., "Rust workspace with spec sync", "TypeScript monorepo", "Polyglot project").
7. **CTA banner**: "Stop documenting in <em>fiction.</em> / Install spec-sync and run your first check in under a minute." Install snippet + `Read the docs` / `Browse 12 languages` actions.
8. **Footer**.

### `/languages` — registry

The structural analog of fledge's `/plugins`. Same controls bar, same grid pattern. **Data source is different: hand-maintained `site/src/data/languages.json`** rather than a GitHub fetch. (Maintenance is low — 12 entries, one PR per added language.)

Page sections:
1. Page-head: eyebrow `Languages`, H1 "Spec-sync speaks <em>your stack.</em>", lede, quick stats (12 languages · auto-detected · one spec format).
2. Controls bar: search box (placeholder "Search by language or extension…"), filter chips for runtime family (Native / Managed / Dynamic / Markup) and detection style (AST / Regex / Hybrid), sort dropdown (Name A-Z / Recently added).
3. 2- or 3-column grid of language cards. Each card:
   - Language name + family tag.
   - Test-file pattern (mono).
   - Short description of what's detected.
   - File extensions chip row (`.ts`, `.tsx`, `.d.ts` excluded etc.).
   - `Read detection rules →` link to per-language page.
4. Bottom panel: "Don't see your language?" → link to the "add a language" GitHub discussion / contribution guide.

**Data shape** (`site/src/data/languages.json`):

```jsonc
[
  {
    "slug": "typescript",
    "name": "TypeScript / JS",
    "family": "managed",
    "detection_style": "ast",
    "extensions": [".ts", ".tsx", ".js", ".jsx", ".mts", ".cts"],
    "test_patterns": [".test.ts", ".spec.ts", ".d.ts"],
    "exports_detected": [
      "export function/class/type/const/enum",
      "re-exports",
      "export * wildcard resolution"
    ],
    "description": "Full AST-based export detection. Resolves re-exports and barrel files. Skips .d.ts and test files."
  },
  // ... 11 more
]
```

### `/languages/{slug}` — per-language page

The structural analog of fledge's `/plugins/{slug}`. Each language gets its own page from a per-language data file at `site/src/data/languages/{slug}.json` (extends the registry entry).

Sections:
1. **Header**: `← Back to languages` breadcrumb; language name (large); family tag; detection-style tag; "since vX.Y" version pill.
2. **Detection rules** — table or list of what counts as an export, including code snippets showing examples.
3. **Test-file patterns** — what's auto-excluded.
4. **Example spec block** — an actual `*.spec.md` file annotated for this language, with the matching source side-by-side.
5. **Caveats / known limitations** — anything language-specific (e.g., "Rust macros generating exports are not detected").
6. **MDX deep-dive** — optional content from `site/src/content/languages/{slug}.mdx` for language-specific docs (long-form patterns, advanced configuration). If no MDX file exists for that slug, this section is hidden.
7. **Related languages** — 3 cards picked by shared `family` (e.g., TypeScript suggests JavaScript, Dart).
8. **Footer CTA** — "Found a bug or want to extend support? File an issue →".

Per-language JSON shape extends the registry entry with:

```jsonc
{
  // all fields from the registry entry, plus:
  "detection_rules": [
    {
      "title": "Top-level export keywords",
      "description": "export function, export class, export const, export type, export enum",
      "example_code": "export function add(a: number, b: number): number { return a + b }"
    },
    // ...
  ],
  "test_pattern_examples": [
    { "pattern": "**/*.test.ts", "explanation": "Auto-excluded" },
    { "pattern": "**/*.d.ts",    "explanation": "Type-only files skipped" }
  ],
  "example_spec_md":   "# Math\n\n## Exports\n\n- `add` ...",
  "example_source":    "export function add(a: number, b: number): number { return a + b }",
  "caveats": [
    "Re-exports from external packages are not followed across package boundaries.",
    "Conditional exports in package.json are not parsed."
  ],
  "related_slugs": ["javascript"],
  "since_version": "v1.0"
}
```

These per-language JSON files are **hand-authored alongside the source language additions** — they live in `site/src/data/languages/` and are checked in. Adding a language to the registry is a small PR that adds two files.

### `/examples`, `/examples/{slug}`, `/docs`, `/docs/{...}`, `/blog`, `/blog/{slug}`

Identical structure to the fledge spec. Different content (examples and seed blog posts are spec-sync specific). Same Astro components, same layouts, same prev/next pager, same color-coded category tags.

Seed walkthroughs (3):
1. "Sync a Rust workspace with spec-sync" — `tag: Rust workspace`
2. "Multi-language repo: TS + Python + Rust" — `tag: Polyglot`
3. "CI gate with the GitHub Action" — `tag: CI / CD`

Seed blog posts (2):
1. **Release** — current version notes (mirrors v1.4.1 post for fledge).
2. **Announce** — "Marketing site rebuild — what's coming and why" (mirrors the fledge counterpart, spec-sync specific).

## Repo layout

Same as fledge with two substitutions:

```
site/
  astro.config.mjs        site: 'https://corvidlabs.github.io', base: '/spec-sync/'
  scripts/
    # No build-plugin-registry (no GH fetch).
    # Languages JSON is hand-maintained and committed.
    validate-languages.ts        # asserts site/src/data/languages.json is schema-valid; runs in CI
    validate-languages.test.ts
    generate-doc-redirects.ts    # same as fledge — generates /spec-sync/<old>.html → /spec-sync/docs/<new>
    generate-doc-redirects.test.ts
  src/
    data/
      languages.json             # registry index; committed (12 entries)
      languages/
        typescript.json          # per-language; committed
        rust.json
        # ... 10 more
    content/
      docs/                      # migrated mdBook
      examples/                  # MDX walkthroughs
      blog/                      # MDX posts
      languages/                 # optional MDX deep-dives (one per slug, if present)
    pages/
      index.astro
      404.astro
      languages/
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
```

Old `docs/` mdBook tree is deleted in the cutover commit alongside `.github/workflows/docs.yml`.

## Build & deploy

`.github/workflows/pages.yml` is identical in shape to fledge's, with three changes:
1. Trigger paths watch `site/**` only — no need to re-deploy weekly for fresh registry data (the languages file is hand-maintained).
2. Drop the weekly cron. Re-add it only if the team wants periodic deploys to pick up dependency updates.
3. Prebuild runs `bun scripts/validate-languages.ts && bun scripts/generate-doc-redirects.ts` instead of `build-plugin-registry`.

## Open questions (must answer before plan)

None — the brainstorm with @0xLeif locked all the same answers as the fledge spec (no mailing list, per-{entity} pages shipped, docs search deferred). The blue palette, Six powers framing, and Languages page slot were all confirmed.

## Success criteria

- All four nav routes live: `/`, `/languages`, `/examples`, `/docs`, `/blog`.
- All 12 languages have a working `/languages/{slug}` page with detection rules, test patterns, and at least one example spec/source pair.
- Migrated docs match the existing mdBook content. Old top-level URLs (`/spec-sync/spec-format.html`, etc.) return either matching content or a 1-step redirect — never a 404.
- `cargo install specsync` snippet copies to clipboard in one click from the home and CTA.
- Lighthouse ≥ 95 on Performance, Accessibility, Best Practices, SEO (mobile + desktop) for `/`, `/languages`, and a representative `/docs/{page}`.
- `prefers-reduced-motion: reduce` produces a static page.
- `bun test` passes (covers `validate-languages.ts` and `generate-doc-redirects.ts`).
- Adding a 13th language requires only a PR adding two JSON files (registry entry + per-language file) — no template changes needed.
