import { readdirSync, statSync, mkdirSync, writeFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const PUBLIC_DIR = join(__dirname, '..', 'public')
const BASE = '/spec-sync/'

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
    // Skip SUMMARY.md and index.md — these don't need redirects
    if (f === 'SUMMARY.md' || f === 'index.md') continue
    const html = f.replace(/\.md$/, '.html')
    const slug = f.replace(/\.md$/, '')
    const newPath = `${BASE}docs/${slug}`.replace(/\/+/g, '/')
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
