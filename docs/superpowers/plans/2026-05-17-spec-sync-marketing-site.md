# spec-sync marketing site — Implementation Plan (diff against fledge)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans. This plan is a **diff** — execute the fledge plan with the substitutions below.

**Goal:** Apply the fledge marketing-site pattern to `CorvidLabs/spec-sync` with sky-blue palette, a Languages registry instead of a Plugin registry, and "Six powers" instead of "Six pillars".

**Base plan:** `docs/superpowers/plans/2026-05-17-marketing-site.md` (the fledge plan). Read it first. **Spec-sync is built by executing every phase of that plan with the substitutions below.**

**Spec:** `docs/superpowers/specs/2026-05-17-spec-sync-marketing-site-design.md`

**Execute when:** after the fledge site ships and the pattern is battle-tested.

**Working branch:** new branch `docs/spec-sync-marketing-site` off `main` in the `CorvidLabs/spec-sync` repo.

---

## Global substitutions (apply everywhere)

| In the fledge plan you'll see | In spec-sync, replace with |
|---|---|
| `fledge` (CLI name) | `spec-sync` / `specsync` (binary is `spec-sync`, crate is `specsync`) |
| `corvidlabs.github.io/fledge` | `corvidlabs.github.io/spec-sync` |
| Astro `base: '/fledge/'` | `base: '/spec-sync/'` |
| `cargo install fledge` | `cargo install specsync` |
| "Get your projects ready to fly" | "Keep your docs honest" |
| Italic-serif accent words: `fly`, `nothing`, `shipped`, `real` | `sync`, `truth`, `honest`, `everywhere` |
| `f` glyph in logo tile | `s` glyph in logo tile |
| Nav link `Plugins` | `Languages` |
| "Six Pillars" + Scaffold/Run/Spec/AI/Ship/Extend | "Six Powers" + Validate/Languages/Generate/Integrate/Cross-refs/Extend |
| `--accent: #ea580c` | `--accent: #0ea5e9` |
| `--accent-bright: #fdba74` | `--accent-bright: #7dd3fc` |
| `--accent-deep: #9a3412` | `--accent-deep: #075985` |
| `--accent-muted: rgba(234, 88, 12, 0.10)` | `--accent-muted: rgba(14, 165, 233, 0.10)` |
| `--accent-glow: rgba(234, 88, 12, 0.15)` | `--accent-glow: rgba(14, 165, 233, 0.18)` |
| `--bg: #0c0a09` | `--bg: #0a1018` |
| `--bg-raised: #18120e` | `--bg-raised: #0e1825` |
| `--border: #2a201a` | `--border: #1c2a3a` |
| `--border-strong: #3a2c22` | `--border-strong: #2a3b4f` |
| `--text-muted: #c2b9b1` | `--text-muted: #b8c5d4` |
| `--text-dim: #9c948d` | `--text-dim: #94a3b8` |
| Favicon gradient stops `#fdba74 → #ea580c → #9a3412` | `#7dd3fc → #0ea5e9 → #075985` |
| Logo tile text color `#1a0f08` | `#08141d` |
| Hero terminal demo (templates init / lanes init / lanes run ci) | `spec-sync check` finding a phantom export, then a `spec-sync generate` follow-up |

Apply these substitutions to **every file the fledge plan creates**, including all CSS custom properties, all Astro components, all page content, all seed walkthroughs, all seed blog posts, and all repository metadata files (`package.json`, `README.md`, etc.).

---

## Phase-by-phase divergences from the fledge plan

### Phases 1, 2, 3 — Scaffold, design system, home page

Execute the fledge plan's Phases 1–3 verbatim with the global substitutions applied. The home page's pillar grid becomes the **Six Powers** grid:

```astro
<Pillar numeral="i."   title="Validate"   command="$ spec-sync check">Bidirectional. Code → spec and spec → code. Errors on drift.</Pillar>
<Pillar numeral="ii."  title="Languages"  command="12 detected">TypeScript, Rust, Go, Python, Swift, Kotlin, Java, C#, Dart, PHP, Ruby, YAML.</Pillar>
<Pillar numeral="iii." title="Generate"   command="$ spec-sync generate">AI-assisted spec generation from existing code. Any provider.</Pillar>
<Pillar numeral="iv."  title="Integrate"  command="vscode | gh actions">VS Code extension + GitHub Action + CLI. Drop into any CI.</Pillar>
<Pillar numeral="v."   title="Cross-refs" command="$ spec-sync graph">Reference exports across repos. Dependency graph included.</Pillar>
<Pillar numeral="vi."  title="Extend"     command="$ spec-sync --config">Custom validators, hooks, output formats. Open file formats.</Pillar>
```

### Phase 4 — Languages registry pipeline (REPLACES fledge's plugin pipeline)

**Skip fledge tasks 4.4–4.6 entirely** (no GitHub fetch — languages are first-party). Keep tasks 4.1–4.3 only if they have analog use; otherwise skip the entire fledge Phase 4 and execute the simpler substitute below.

**Substitute task — Language registry data + validator:**

- **Files:**
  - Create: `site/scripts/validate-languages.ts`
  - Create: `site/scripts/validate-languages.test.ts`
  - Create: `site/src/data/languages.json` (12 entries — see spec for shape)
  - Create: `site/src/data/languages/{slug}.json` ×12 (per-language deep data)

- **Steps:**
  1. Write `validate-languages.ts` exporting `validateRegistry(json): {ok: true} | {ok: false, errors: string[]}` that checks: required keys present, slug matches filename, family ∈ `{native, managed, dynamic, markup}`, detection_style ∈ `{ast, regex, hybrid}`, extensions array non-empty, per-language file exists for every registry entry, every per-language file has a corresponding registry entry.
  2. Write `validate-languages.test.ts` with at least: happy path, missing field, slug/filename mismatch, dangling per-language file.
  3. Implement; tests pass.
  4. Hand-author `site/src/data/languages.json` with all 12 entries (TypeScript, Rust, Go, Python, Swift, Kotlin, Java, C#, Dart, PHP, Ruby, YAML — populate from the `spec-sync` README's Supported Languages table).
  5. Hand-author `site/src/data/languages/{slug}.json` ×12 with detection_rules, test_pattern_examples, example_spec_md, example_source, caveats, related_slugs, since_version (per the spec shape).
  6. Wire `prebuild` in `package.json` to `bun scripts/validate-languages.ts && bun scripts/generate-doc-redirects.ts`. The validator runs first and exits non-zero on schema errors — so a malformed registry fails the build.
  7. Commit.

This task is much smaller than fledge's Phase 4 — it's data entry plus a schema validator. Estimated ~2 hours.

### Phase 5 — Plugin pages → Language pages

Rename and adapt:
- `site/src/components/PluginCard.astro` → `LanguageCard.astro`. Shows: language name, family tag (instead of trust tier), detection-style tag, test-pattern preview, description, "Read detection rules →" link.
- `site/src/pages/plugins/index.astro` → `site/src/pages/languages/index.astro`. Search by name / extension / family. Filter chips for **family** (`Native / Managed / Dynamic / Markup`) and **detection style** (`AST / Regex / Hybrid`).
- `site/src/pages/plugins/[slug].astro` → `site/src/pages/languages/[slug].astro`. Renders per-language JSON. Sections from the spec: header → detection rules → test patterns → example spec block → caveats → optional MDX deep-dive → related languages → footer CTA.
- Per-language pages may pull additional long-form content from `site/src/content/languages/{slug}.mdx` if a file exists for that slug; otherwise that section is hidden.

### Phase 6 — Examples

Same structure, different seed content. Three seed walkthroughs:
1. `site/src/content/examples/rust-workspace.mdx` (tag: `Rust workspace`)
2. `site/src/content/examples/polyglot.mdx` (tag: `Polyglot`)
3. `site/src/content/examples/ci-gate.mdx` (tag: `CI / CD`)

`tag` enum in the content collection schema (`site/src/content/config.ts`) is `['Rust workspace', 'Polyglot', 'CI / CD', 'VS Code', 'GitHub Action', 'Languages']` instead of fledge's enum.

### Phase 7 — Docs migration

Mechanically identical, but the source mdBook tree is in `CorvidLabs/spec-sync` (not in fledge). Mirror the migration table from the fledge plan's Task 7.2 against spec-sync's `docs/src/` files. Section names ("Getting started", "Spec format", "Reference", "Integrations") may differ — generate them from spec-sync's existing `SUMMARY.md`.

### Phase 8 — Blog

Same structure. Two seed posts:
1. **Release** post for the current `specsync` crate version (`site/src/content/blog/<version>-release.mdx`, `category: release`).
2. **Announce** post — `site/src/content/blog/welcome.mdx`, `category: announce`, mirrors the fledge welcome post.

### Phase 9 — Build & deploy + cutover

Identical to fledge **except**:
- `.github/workflows/pages.yml`: **remove the weekly cron** (no upstream registry to refresh). Keep the `push` + `workflow_dispatch` triggers.
- Prebuild step is `bun scripts/validate-languages.ts && bun scripts/generate-doc-redirects.ts` instead of the registry fetcher.
- `generate-doc-redirects.ts` is identical except for the `BASE` constant: `/spec-sync/` instead of `/fledge/`.
- Cutover deletes the existing `docs/` (mdBook) tree in `CorvidLabs/spec-sync`.

### Phase 10 — A11y + Lighthouse polish

Identical. Run axe + Lighthouse against `/spec-sync/`, `/spec-sync/languages`, and a sample `/spec-sync/docs/{page}`.

---

## Self-review

- **Spec coverage:** every section of the spec-sync spec maps to a fledge-plan phase via the substitutions above, except the `/languages` data pipeline — covered by the substitute Phase 4 task.
- **No placeholders:** every substitution is concrete (specific colors, specific copy, specific file paths). The 12-language data entry is a real authoring task, not a TBD.
- **Type consistency:** `LanguageCard` replaces `PluginCard` consistently across components, layouts, and pages. `validateRegistry` is referenced once in `validate-languages.ts` and once in its test file.
- **Order risk:** none — substitutions are mechanical.
